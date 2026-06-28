use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use moonveil_core::{AesGcmCipher, Cipher, Packet};

fn bench_packet_serialize(c: &mut Criterion) {
    let mut packets = Vec::with_capacity(1000);
    for i in 0..1000u64 {
        packets.push(Packet::new(i, vec![i as u8; 32]));
    }

    c.bench_function("packet_serialize/1000", |b| {
        b.iter(|| {
            for pkt in &packets {
                let bytes = black_box(bincode::serialize(pkt).unwrap());
                black_box(bytes);
            }
        })
    });
}

fn bench_packet_deserialize(c: &mut Criterion) {
    let packets: Vec<Packet> = (0..1000u64)
        .map(|i| Packet::new(i, vec![i as u8; 32]))
        .collect();

    let serialized: Vec<Vec<u8>> = packets
        .iter()
        .map(|pkt| bincode::serialize(pkt).unwrap())
        .collect();

    c.bench_function("packet_deserialize/1000", |b| {
        b.iter(|| {
            for bytes in &serialized {
                let pkt: Packet = black_box(bincode::deserialize(bytes).unwrap());
                black_box(pkt);
            }
        })
    });
}

fn bench_aes_encrypt(c: &mut Criterion) {
    let key = [7u8; 32];
    let cipher = AesGcmCipher::new(key);

    let payload = vec![42u8; 1024];

    c.bench_with_input(BenchmarkId::new("aes_gcm_encrypt", "1kb"), &payload, |b, data| {
        b.iter(|| {
            let encrypted = black_box(cipher.encrypt(black_box(data)).unwrap());
            black_box(encrypted);
        })
    });
}

fn bench_aes_decrypt(c: &mut Criterion) {
    let key = [7u8; 32];
    let cipher = AesGcmCipher::new(key);

    let payload = vec![42u8; 1024];
    let encrypted = cipher.encrypt(payload.as_slice()).unwrap();

    c.bench_function("aes_gcm_decrypt/1kb", |b| {
        b.iter(|| {
            let decrypted = black_box(cipher.decrypt(black_box(encrypted.as_slice())).unwrap());
            black_box(decrypted);
        })
    });
}

criterion_group!(
    benches,
    bench_packet_serialize,
    bench_packet_deserialize,
    bench_aes_encrypt,
    bench_aes_decrypt
);
criterion_main!(benches);
