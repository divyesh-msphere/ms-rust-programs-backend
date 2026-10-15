// Hand-written wiring for the buf-generated, file_per_package output of the
// `buf.build/anthropics/buffa` and `buf.build/connectrpc/rust` remote
// plugins (see ../../../../buf.gen.yaml) — each plugin gets its own `out:`
// dir, so the two module trees are stitched together here rather than by a
// packaging plugin.
pub mod buffa {
    pub mod programs {
        pub mod v1 {
            include!("buffa/programs.v1.rs");
        }
    }
}

pub mod connect {
    pub mod programs {
        pub mod v1 {
            include!("connect/programs.v1.rs");
        }
    }
}
