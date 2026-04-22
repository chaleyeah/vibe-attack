use crossbeam_channel::{Receiver, Sender};

fn try_send_drop_oldest<T>(tx: &Sender<T>, rx: &Receiver<T>, item: T) -> Result<(), T> {
    hd_linux_voice::vad::try_send_drop_oldest(tx, rx, item)
}

#[test]
fn drop_oldest_keeps_newest_items() {
    let (tx, rx) = crossbeam_channel::bounded::<u32>(3);

    for i in 0..3 {
        try_send_drop_oldest(&tx, &rx, i).expect("initial fill must succeed");
    }

    // Queue is full (0,1,2). Inserting 3 should drop 0.
    try_send_drop_oldest(&tx, &rx, 3).expect("drop-oldest insert must succeed");

    let got: Vec<u32> = (0..3).map(|_| rx.recv().unwrap()).collect();
    assert_eq!(got, vec![1, 2, 3], "oldest item must be dropped");
}

#[test]
fn drop_oldest_never_exceeds_capacity() {
    let (tx, rx) = crossbeam_channel::bounded::<u32>(2);

    for i in 0..10 {
        try_send_drop_oldest(&tx, &rx, i).expect("bounded drop-oldest must succeed");
        assert!(
            rx.len() <= 2,
            "receiver len must never exceed capacity, got {}",
            rx.len()
        );
    }
}

