use sqlite_vec::sqlite3_vec_init;

fn main() {
    unsafe {
        sqlite3_vec_init();   
    }
}
