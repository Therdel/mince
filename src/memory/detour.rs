use anyhow::{Result, anyhow};
use keystone::*;
use region::Protection;

/// Order of detour and patched-code execution.
#[derive(Clone, Copy)]
pub enum DetourOrder {
    /// Detour is executed first, then patched-code.
    DetourBefore,
    /// Patched-code is executed first, then detour.
    DetourAfter
}

/// Patching of opcodes to "insert" a method call.
/// Works by patching a relative jump to a heap-allocated code buffer (trampoline)
/// that executes the patched-over code and calls a method on a given instance
pub struct DetourToMethod {
    patch_region: *mut [u8],
    old_patched_code: Vec<u8>,
    // /// type-erased Rc keeps instance alive
    // #[allow(unused)] erased_instance: Box<dyn Drop>,
    /// trampoline code buffer
    #[allow(unused)] trampoline: Vec<u8>,
    /// holds trampoline execute permissions during lifetime
    #[allow(unused)] trampoline_mem_protection_guard: region::ProtectGuard,
    /// holds patch_region's execute permission during lifetime
    #[allow(unused)] patch_region_mem_protection_guard: region::ProtectGuard,
}

impl DetourToMethod {
    pub unsafe fn install<T>(patch_region: *mut [u8], instance: &T, method: fn(&T), order: DetourOrder) -> Result<Self> {
        // TODO: Do central event dispatching with callback-types (big enum with values and stuff. Would centralize all hussle and keep features clean.)
            // TODO: put current values (like freshly set aim-angles or aim-angles to-set) as references into enum
        let patch_region: &mut [u8] = unsafe {&mut *patch_region};
        if patch_region.len() < 5 {
            return Err(anyhow!("Tried to detour with code patch of len={}", patch_region.len()))
        }

        let keystone =
            Keystone::new(Arch::X86, Mode::MODE_32)
            .map_err(|err| anyhow!("Could not initialize Keystone engine: {err}"))?;
        keystone
            .option(OptionType::SYNTAX, OptionValue::SYNTAX_NASM)
            .map_err(|err| anyhow!("Could not set option to nasm syntax: {err}"))?;

        let old_patched_code = patch_region.to_vec();
        let jmp_back_target_address = patch_region.as_ptr_range().start as usize + patch_region.len();

        let instance_address = instance as *const _ as usize;
        let method_address = method as usize;
        let trampoline = Self::build_trampoline(&keystone, instance_address, method_address, order, old_patched_code.as_slice(), jmp_back_target_address)?;
        let trampoline_slice = &trampoline[..];
        let trampoline_mem_protection_guard = unsafe {
            region::protect_with_handle(trampoline_slice.as_ptr(),
                trampoline_slice.len(),
                Protection::READ_WRITE_EXECUTE)?
        };

        // assemble detour jmp
        let address_of_detour_jmp_instruction = patch_region.as_ptr() as usize;
        let address_of_trampoline = trampoline_slice.as_ptr() as usize;
        let detour_jmp_code =
            Self::assemble_relative_jump(&keystone, address_of_detour_jmp_instruction, address_of_trampoline)?;

        // write detour jmp into start of patch region, fill the rest with NOPs
        const NOP: u8 = 0x90;
        let patch_region_mem_protection_guard = unsafe {
            region::protect_with_handle(patch_region.as_ptr(),
            patch_region.len(),
            Protection::READ_WRITE_EXECUTE)?
        };
        let (jmp_region, nop_region) = patch_region.split_at_mut(detour_jmp_code.len());
        jmp_region.copy_from_slice(&detour_jmp_code);
        nop_region.fill(NOP);

        Ok(Self {patch_region, old_patched_code, trampoline, patch_region_mem_protection_guard, trampoline_mem_protection_guard})
    }

    fn build_trampoline(keystone: &Keystone, instance_address: usize, method_address: usize,
                        order: DetourOrder, old_code: &[u8],
                        jmp_back_target_address: usize) -> Result<Vec<u8>> {
        let asm = format!(
            "push eax
            push ebx
            push ecx
            push edx
            push esi
            push edi
            pushfd
            mov eax, {instance_address:#x}
            mov esi, {method_address:#x}
            push    eax
            call    esi
            pop     eax
            popfd
            pop edi
            pop esi
            pop edx
            pop ecx
            pop ebx
            pop eax");

        let method_calling_code = keystone.asm(asm, 0)
            .map_err(|err| anyhow!("Could not assemble trampoline: {err}"))?
            .bytes;
        
        // allocate trampoline
        const CODE_LENGTH_JMP_INSTRUCTION: usize = 5;
        let trampoline_code_size = method_calling_code.len() + old_code.len() + CODE_LENGTH_JMP_INSTRUCTION;
        let mut trampoline = Vec::<u8>::new();
        trampoline.reserve(trampoline_code_size);
        
        // write old code and method calling code, depending on detour order
        match order {
            DetourOrder::DetourBefore => { 
                trampoline.extend_from_slice(method_calling_code.as_slice());
                trampoline.extend_from_slice(old_code);
            },
            DetourOrder::DetourAfter => {
                trampoline.extend_from_slice(old_code);
                trampoline.extend_from_slice(method_calling_code.as_slice());
            }
        }

        // jump back to the instruction after the detour that calls the trampoline
        let address_of_jmp_back_instruction =
            trampoline.as_ptr() as usize + method_calling_code.len() + old_code.len();
        let jmp_back_code =
            Self::assemble_relative_jump(keystone, address_of_jmp_back_instruction, jmp_back_target_address)?;
        trampoline.extend_from_slice(jmp_back_code.as_slice());

        Ok(trampoline)
    }

    fn assemble_relative_jump(keystone: &Keystone, address_jmp_instruction: usize, address_target: usize) -> Result<Vec<u8>> {
        let asm = format!("jmp {address_target:#x}");
        let jmp_code = keystone.asm(asm, address_jmp_instruction as _)
            .map_err(|err| anyhow!("Could not assemble jmp: {err}"))?
            .bytes;

        Ok(jmp_code)
    }
}

impl Drop for DetourToMethod {
    fn drop(&mut self) {
        let patch_region: &mut [u8] = unsafe {&mut *self.patch_region};
        // patch_region is still writable as its protection guard is still alive in self
        patch_region.copy_from_slice(&self.old_patched_code);
    }
}
