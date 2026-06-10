# Rust as the primary runtime

AuroraPulse will use Rust as its primary runtime, while the existing Python implementation remains a prototype and reference during migration. We chose this because AuroraPulse is aiming for a dependable local daily assistant with a long-lived CLI and future voice or background runtime, where Rust gives us a stronger foundation for performance, packaging, and operational stability than keeping two equal implementation tracks.
