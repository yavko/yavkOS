super::stream_processor_task!(u8, 2048);

pub async fn process() {
    let mut stream = TaskStream::new();
    let mouse = crate::mouse::get().expect("mouse should be initialized by now");
    while let Some(packet) = stream.next().await {
        mouse.add(packet).await;
    }
}
