// ========== 动态GPU缓冲区模块 ==========
// 封装 wgpu::Buffer，支持在运行时动态扩容
// 当立方体数量增加时，重新分配更大的缓冲区

use iced::wgpu;

// 自定义的动态缓冲区容器（支持resize操作）
pub struct Buffer {
    pub raw: wgpu::Buffer,                     // 实际的wgpu缓冲区对象
    label: &'static str,                       // 调试标签（用于GPU调试工具识别此缓冲区）
    size: u64,                                 // 当前缓冲区容量（字节）
    usage: wgpu::BufferUsages,                 // 缓冲区用途标志（如VERTEX、COPY_DST等）
}

impl Buffer {                                  // Buffer的方法实现
    // 创建新的动态缓冲区
    // label: GPU调试标签
    // size: 初始容量（字节）
    // usage: 缓冲区用途（位标志组合）
    pub fn new(
        device: &wgpu::Device,                 // wgpu设备
        label: &'static str,                   // 调试标签
        size: u64,                             // 初始大小
        usage: wgpu::BufferUsages,             // 用途标志
    ) -> Self {
        Self {
            raw: device.create_buffer(&wgpu::BufferDescriptor { // 创建实际的GPU缓冲区
                label: Some(label),            // 设置调试标签
                size,                          // 设置大小
                usage,                         // 设置用途
                mapped_at_creation: false,     // 不映射到CPU（通过队列传输数据）
            }),
            label,                             // 保存标签（重建时复用）
            size,                              // 保存容量
            usage,                             // 保存用途标志
        }
    }

    // 扩容缓冲区（仅在需要更大空间时重新分配）
    // 策略：只在new_size > size时才重建，不会缩小
    pub fn resize(&mut self, device: &wgpu::Device, new_size: u64) {
        if new_size > self.size {              // 只有新容量超过旧容量才重建
            self.raw = device.create_buffer(&wgpu::BufferDescriptor { // 创建更大的新缓冲区
                label: Some(self.label),       // 复用标签
                size: new_size,                // 使用新容量
                usage: self.usage,             // 复用用途标志
                mapped_at_creation: false,
            });

            self.size = new_size;              // 更新记录的容量值
        }
    }
}
