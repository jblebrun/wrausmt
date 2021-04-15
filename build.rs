fn main() {
    println!("cargo:rerun-if-changed=codegen/master_secondary_ops_list.csv");
    println!("cargo:rerun-if-changed=codegen/master_ops_list.csv");
    codegen::parse().unwrap();
}
