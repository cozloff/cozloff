#[cfg(feature = "real-dpdk")]
mod real {
    use std::{
        ffi::CString,
        os::raw::{c_char, c_int},
    };

    unsafe extern "C" {
        fn rte_eal_init(argc: c_int, argv: *mut *mut c_char) -> c_int;
        fn rte_eal_cleanup() -> c_int;
        fn rte_eth_dev_count_avail() -> u16;
    }

    pub struct DpdkRuntime {
        port_count: u16,
    }

    impl DpdkRuntime {
        pub fn init(args: &[String]) -> Result<Self, String> {
            let c_args = args
                .iter()
                .map(|arg| CString::new(arg.as_str()).map_err(|_| format!("argument contains NUL byte: {arg:?}")))
                .collect::<Result<Vec<_>, _>>()?;
            let mut argv = c_args
                .iter()
                .map(|arg| arg.as_ptr().cast_mut())
                .collect::<Vec<_>>();

            let result = unsafe { rte_eal_init(argv.len() as c_int, argv.as_mut_ptr()) };
            if result < 0 {
                return Err("rte_eal_init failed".to_string());
            }

            Ok(Self {
                port_count: unsafe { rte_eth_dev_count_avail() },
            })
        }

        pub fn port_count(&self) -> u16 {
            self.port_count
        }
    }

    impl Drop for DpdkRuntime {
        fn drop(&mut self) {
            let _ = unsafe { rte_eal_cleanup() };
        }
    }
}

#[cfg(not(feature = "real-dpdk"))]
mod stub {
    #[derive(Debug)]
    pub struct DpdkRuntime;

    impl DpdkRuntime {
        pub fn init(_args: &[String]) -> Result<Self, String> {
            Err(
                "built without real DPDK support. Rebuild with `cargo run --features real-dpdk -- <EAL args>` after installing DPDK."
                    .to_string(),
            )
        }

        pub fn port_count(&self) -> u16 {
            0
        }
    }
}

#[cfg(feature = "real-dpdk")]
pub use real::DpdkRuntime;

#[cfg(not(feature = "real-dpdk"))]
pub use stub::DpdkRuntime;

#[cfg(test)]
mod tests {
    use super::DpdkRuntime;

    #[cfg(not(feature = "real-dpdk"))]
    #[test]
    fn stub_mode_explains_how_to_enable_real_dpdk() {
        let args = vec!["cozloff-dpdk".to_string()];
        let error = DpdkRuntime::init(&args).unwrap_err();

        assert!(error.contains("--features real-dpdk"));
    }
}
