use parallel_processor::buckets::concurrent::BucketsThreadDispatcher;
use parallel_processor::buckets::MultiThreadBuckets;
use parallel_processor::lock_free_binary_writer::LockFreeBinaryWriter;
use parallel_processor::memory_data_size::MemoryDataSize;
use parallel_processor::memory_fs::file::internal::MemoryFileMode;
use std::path::PathBuf;
use std::sync::Arc;

pub fn writing_test(cpu_count: usize) {
    println!("Init fs...");
    parallel_processor::memory_fs::MemoryFs::init(MemoryDataSize::from_gibioctets(64), 4096, 3, 0);

    println!("Init buckets...");
    let files = Arc::new(MultiThreadBuckets::<LockFreeBinaryWriter>::new(
        256,
        &(PathBuf::from("/tmp/null"), MemoryFileMode::DiskOnly),
        None,
    ));

    for i in 0..cpu_count {
        let files = files.clone();
        std::thread::spawn(move || {
            let mut thread = BucketsThreadDispatcher::<_, [u8]>::new(
                MemoryDataSize::from_kibioctets(64),
                &files,
            );
            loop {
                for i in 0..256 {
                    thread.add_element(i, &(), &[1, 2, 3, 4])
                }
            }
        });
    }
}
