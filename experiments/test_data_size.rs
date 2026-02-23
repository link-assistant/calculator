// Test data size unit conversion in the calculator
// Run with: cargo run --example test_data_size

fn main() {
    // This would test the calculator binary directly
    // Let's test via the library API instead
    println!("Data size unit conversion tests");
    println!("================================");
    
    // Test: 741 KB as MB
    // Expected: 0.741 MB
    
    // Test: 741 KB as mebibytes  
    // Expected: ~0.7069 MiB
    
    // Test: 741 KiB as MiB
    // Expected: 741/1024 ≈ 0.7236 MiB
    
    println!("Expected results:");
    println!("741 KB as MB = {:.10} MB", 741.0_f64 * 1000.0 / 1_000_000.0);
    println!("741 KB as MiB = {:.10} MiB", 741.0_f64 * 1000.0 / 1_048_576.0);
    println!("741 KiB as MiB = {:.10} MiB", 741.0_f64 * 1024.0 / 1_048_576.0);
    println!("741 KiB as MiB = {:.10} MiB (as 741/1024)", 741.0_f64 / 1024.0);
}
