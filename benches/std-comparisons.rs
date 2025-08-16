use criterion::{BenchmarkId, Criterion, Throughput, black_box, criterion_group, criterion_main};
use sakurai::*;
use std::collections::{BTreeMap, HashMap as StdHashMap, VecDeque};

fn bench_ringbuf(c: &mut Criterion) {
    let mut group = c.benchmark_group("ringbuf");
    group.bench_function("sakurai:ring_push", |b| {
        let buffer = RingBuffer::<u64, 1024>::new();
        let mut counter = 0u64;
        b.iter(|| {
            let _ = buffer.push(black_box(counter));
            counter = counter.wrapping_add(1);
        });
    });

    group.bench_function("std:vec_deque_push", |b| {
        let mut deque = VecDeque::with_capacity(1024);
        let mut counter = 0u64;
        b.iter(|| {
            if deque.len() >= 1023 {
                deque.pop_front();
            }
            deque.push_back(black_box(counter));
            counter = counter.wrapping_add(1);
        });
    });

    group.bench_function("sakurai:ring_pop", |b| {
        let buffer = RingBuffer::<u64, 1024>::new();
        for i in 0..512 {
            let _ = buffer.push(i);
        }
        b.iter(|| {
            let _ = buffer.pop();
            let _ = buffer.push(black_box(0));
        });
    });

    group.bench_function("std:vec_deque_pop", |b| {
        let mut deque = VecDeque::with_capacity(1024);
        for i in 0..512 {
            deque.push_back(i);
        }
        b.iter(|| {
            let _ = deque.pop_front();
            deque.push_back(black_box(0));
        });
    });

    group.finish();
}

fn bench_stack(c: &mut Criterion) {
    let mut group = c.benchmark_group("stack");
    group.bench_function("sakurai:stack_push", |b| {
        let mut stack = Stack::<u64, 1024>::new();
        let mut counter = 0u64;
        b.iter(|| {
            if stack.is_full() {
                stack.clear();
            }
            let _ = stack.push(black_box(counter));
            counter = counter.wrapping_add(1);
        });
    });

    group.bench_function("std:vec_push", |b| {
        let mut vec = Vec::with_capacity(1024);
        let mut counter = 0u64;
        b.iter(|| {
            if vec.len() >= 1024 {
                vec.clear();
            }
            vec.push(black_box(counter));
            counter = counter.wrapping_add(1);
        });
    });

    group.bench_function("sakurai:stack_pop", |b| {
        let mut stack = Stack::<u64, 1024>::new();
        for i in 0..512 {
            let _ = stack.push(i);
        }
        b.iter(|| {
            let _ = stack.pop();
            let _ = stack.push(black_box(0));
        });
    });

    group.bench_function("std:vec_pop", |b| {
        let mut vec = Vec::with_capacity(1024);
        for i in 0..512 {
            vec.push(i);
        }
        b.iter(|| {
            let _ = vec.pop();
            vec.push(black_box(0));
        });
    });

    group.finish();
}

fn bench_spsc_queue(c: &mut Criterion) {
    let mut group = c.benchmark_group("queue");
    group.bench_function("sakurai:queue_throughput", |b| {
        let queue = Queue::<u64, 1024>::new();
        let (mut producer, mut consumer) = queue.split();
        let mut counter = 0u64;
        b.iter(|| {
            if producer.push(black_box(counter)).is_ok() {
                let _ = consumer.pop();
            }
            counter = counter.wrapping_add(1);
        });
    });

    group.bench_function("std:vec_deque_throughput", |b| {
        let mut deque = VecDeque::with_capacity(1024);
        let mut counter = 0u64;
        b.iter(|| {
            if deque.len() >= 1023 {
                let _ = deque.pop_front();
            }
            deque.push_back(black_box(counter));
            counter = counter.wrapping_add(1);
        });
    });

    group.finish();
}

fn bench_hash_map(c: &mut Criterion) {
    let mut group = c.benchmark_group("hashmap");
    for size in [100, 500, 1000].iter() {
        group.throughput(Throughput::Elements(*size as u64));
        group.bench_with_input(
            BenchmarkId::new("sakurai:hashmap_insert", size),
            size,
            |b, &size| {
                b.iter(|| {
                    let mut map = HashMap::<u32, u64, 2048>::new();
                    for i in 0..size {
                        let _ = map.insert(black_box(i), black_box(i as u64 * 2));
                    }
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("std:hashmap_insert", size),
            size,
            |b, &size| {
                b.iter(|| {
                    let mut map = StdHashMap::with_capacity(2048);
                    for i in 0..size {
                        map.insert(black_box(i), black_box(i as u64 * 2));
                    }
                });
            },
        );
    }

    group.bench_function("sakurai:hashmap_lookup", |b| {
        let mut map = HashMap::<u32, u64, 2048>::new();
        for i in 0..1000 {
            let _ = map.insert(i, i as u64 * 2);
        }
        let mut counter = 0u32;
        b.iter(|| {
            let _ = map.get(&black_box(counter % 1000));
            counter = counter.wrapping_add(1);
        });
    });

    group.bench_function("std:hashmap_lookup", |b| {
        let mut map = StdHashMap::with_capacity(2048);
        for i in 0..1000 {
            map.insert(i, i as u64 * 2);
        }
        let mut counter = 0u32;
        b.iter(|| {
            let _ = map.get(&black_box(counter % 1000));
            counter = counter.wrapping_add(1);
        });
    });

    group.finish();
}

fn bench_fixed_vec(c: &mut Criterion) {
    let mut group = c.benchmark_group("fixedvec");
    group.bench_function("sakurai:fixedvec_push", |b| {
        let mut vec = FixedVec::<u64, 1024>::new();
        let mut counter = 0u64;
        b.iter(|| {
            if vec.is_full() {
                vec.clear();
            }
            let _ = vec.push(black_box(counter));
            counter = counter.wrapping_add(1);
        });
    });

    group.bench_function("std:vec_push", |b| {
        let mut vec = Vec::with_capacity(1024);
        let mut counter = 0u64;
        b.iter(|| {
            if vec.len() >= 1024 {
                vec.clear();
            }
            vec.push(black_box(counter));
            counter = counter.wrapping_add(1);
        });
    });

    group.bench_function("sakurai:fixedvec_index", |b| {
        let mut vec = FixedVec::<u64, 1024>::new();
        for i in 0..1024 {
            let _ = vec.push(i);
        }
        let mut counter = 0usize;
        b.iter(|| {
            let _ = black_box(vec[counter % 1024]);
            counter = counter.wrapping_add(1);
        });
    });

    group.bench_function("std:vec_index", |b| {
        let mut vec = Vec::with_capacity(1024);
        for i in 0..1024 {
            vec.push(i);
        }
        let mut counter = 0usize;
        b.iter(|| {
            let _ = black_box(vec[counter % 1024]);
            counter = counter.wrapping_add(1);
        });
    });

    group.finish();
}

fn bench_memory_layout(c: &mut Criterion) {
    let mut group = c.benchmark_group("mem_layout");
    // cache efficient?
    group.bench_function("sakurai:fixedvec_sequential", |b| {
        let mut vec = FixedVec::<u64, 1024>::new();
        for i in 0..1024 {
            let _ = vec.push(i);
        }
        b.iter(|| {
            let mut sum = 0u64;
            for item in vec.iter() {
                sum = sum.wrapping_add(*item);
            }
            black_box(sum);
        });
    });

    group.bench_function("std:vec_sequential", |b| {
        let mut vec = Vec::with_capacity(1024);
        for i in 0..1024 {
            vec.push(i);
        }
        b.iter(|| {
            let mut sum = 0u64;
            for item in &vec {
                sum = sum.wrapping_add(*item);
            }
            black_box(sum);
        });
    });

    group.finish();
}

fn bench_concurrency(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrency");
    group.bench_function("sakurai:spsc_ping_pong", |b| {
        use std::sync::Arc;
        use std::thread;
        b.iter(|| {
            let queue = Arc::new(Queue::<u64, 1024>::new());
            let queue_clone = queue.clone();
            let producer = thread::spawn(move || {
                let (mut prod, _) = queue_clone.split();
                for i in 0..100 {
                    while prod.push(i).is_err() {
                        thread::yield_now();
                    }
                }
            });
            let consumer = thread::spawn(move || {
                let (_, mut cons) = queue.split();
                let mut received = 0;
                while received < 100 {
                    if cons.pop().is_ok() {
                        received += 1;
                    } else {
                        thread::yield_now();
                    }
                }
            });
            producer.join().unwrap();
            consumer.join().unwrap();
        });
    });

    group.finish();
}

fn bench_btree(c: &mut Criterion) {
    let mut group = c.benchmark_group("btree");

    group.bench_function("sakurai:btree_lookup", |b| {
        let mut tree = BTree::<u32, u64, 128>::new();
        for i in 0..128 {
            let _ = tree.insert(i, i as u64 * 2);
        }
        let mut counter = 0u32;

        b.iter(|| {
            let _ = tree.get(&black_box(counter % 128));
            counter = counter.wrapping_add(1);
        });
    });

    group.bench_function("std:btree_lookup", |b| {
        let mut tree = BTreeMap::new();
        for i in 0..128 {
            tree.insert(i, i as u64 * 2);
        }
        let mut counter = 0u32;

        b.iter(|| {
            let _ = tree.get(&black_box(counter % 128));
            counter = counter.wrapping_add(1);
        });
    });

    group.bench_function("sakurai:btree_iteration", |b| {
        let mut tree = BTree::<u32, u64, 128>::new();
        for i in 0..128 {
            let _ = tree.insert(i, i as u64 * 2);
        }

        b.iter(|| {
            let mut sum = 0u64;
            for (_, value) in tree.iter() {
                sum = sum.wrapping_add(*value);
            }
            black_box(sum);
        });
    });

    group.bench_function("std:btree_iteration", |b| {
        let mut tree = BTreeMap::new();
        for i in 0..128 {
            tree.insert(i, i as u64 * 2);
        }

        b.iter(|| {
            let mut sum = 0u64;
            for (_, value) in &tree {
                sum = sum.wrapping_add(*value);
            }
            black_box(sum);
        });
    });

    group.bench_function("sakurai:btree_binarysearch", |b| {
        let mut tree = BTree::<u32, u64, 128>::new();
        for i in 0..128 {
            let _ = tree.insert(i * 2, i as u64);
        }

        let mut counter = 0u32;
        b.iter(|| {
            let _ = tree.get(&black_box(counter % 128));
            counter = counter.wrapping_add(1);
        });
    });

    group.bench_function("std:btree_search", |b| {
        let mut tree = BTreeMap::new();
        for i in 0..128 {
            tree.insert(i * 2, i as u64);
        }

        let mut counter = 0u32;
        b.iter(|| {
            let _ = tree.get(&black_box(counter % 128));
            counter = counter.wrapping_add(1);
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_ringbuf,
    bench_stack,
    bench_spsc_queue,
    bench_hash_map,
    bench_fixed_vec,
    bench_btree,
    bench_memory_layout,
    bench_concurrency,
);
criterion_main!(benches);
