use raw_cpuid::CpuId;

#[derive(Debug, Clone, Copy, Default)]
pub struct CpuFeatures {
    pub sse: bool,
    pub avx: bool,
    pub avx2: bool,
    pub xsave: bool,
    pub xsaveopt: bool,
    pub smep: bool,
    pub smap: bool,
}

impl CpuFeatures {
    pub fn detect() -> Self {
        let cpuid = CpuId::new();
        let mut features = CpuFeatures::default();

        if let Some(fi) = cpuid.get_feature_info() {
            features.sse = fi.has_sse();
            features.avx = fi.has_avx();
            features.xsave = fi.has_xsave();
        }

        if let Some(ext) = cpuid.get_extended_feature_info() {
            features.avx2 = ext.has_avx2();
            features.smep = ext.has_smep();
            features.smap = ext.has_smap();
        }

        if let Some(xsave) = cpuid.get_extended_state_info() {
            features.xsaveopt = xsave.has_xsaveopt();
        }

        features
    }
}

pub fn init_fpu(features: &CpuFeatures) {
    use x86_64::registers::control::{Cr0, Cr0Flags, Cr4, Cr4Flags};
    
    unsafe {
        // Enable SSE/FXSR
        if features.sse {
            let mut cr4 = Cr4::read();
            cr4.insert(Cr4Flags::OSFXSR);
            cr4.insert(Cr4Flags::OSXMMEXCPT_ENABLE);
            Cr4::write(cr4);
            
            let mut cr0 = Cr0::read();
            cr0.remove(Cr0Flags::EMULATE_COPROCESSOR);
            cr0.insert(Cr0Flags::MONITOR_COPROCESSOR);
            Cr0::write(cr0);
        }

        // Enable XSAVE
        if features.xsave {
            let mut cr4 = Cr4::read();
            cr4.insert(Cr4Flags::OSXSAVE);
            Cr4::write(cr4);

            // Enable AVX and SSE in XCR0
            if features.avx {
                use x86_64::registers::xcontrol::{XCr0, XCr0Flags};
                let mut xcr0 = XCr0::read();
                xcr0.insert(XCr0Flags::X87);
                xcr0.insert(XCr0Flags::SSE);
                xcr0.insert(XCr0Flags::AVX);
                XCr0::write(xcr0);
            }
        }
    }
}

pub fn init_security(features: &CpuFeatures) {
    use x86_64::registers::model_specific::{Efer, EferFlags};
    use x86_64::registers::control::{Cr4, Cr4Flags};
    
    unsafe {
        // Enable No-Execute (NX)
        Efer::update(|flags| flags.insert(EferFlags::NO_EXECUTE_ENABLE));
        crate::println!("CPU Security: NX enabled");

        let mut cr4_flags = Cr4::read();
        if features.smep {
            cr4_flags.insert(Cr4Flags::SUPERVISOR_MODE_EXECUTION_PROTECTION);
            crate::println!("CPU Security: SMEP enabled");
        }
        if features.smap {
            cr4_flags.insert(Cr4Flags::SUPERVISOR_MODE_ACCESS_PREVENTION);
            crate::println!("CPU Security: SMAP enabled");
        }
        Cr4::write(cr4_flags);
    }
}
