# Dọn Máy

App desktop cho Windows, viết bằng **Rust + Tauri 2 + React**. App dùng để theo dõi và dọn dẹp máy tính, có AI chạy ngay trên máy qua [Ollama](https://ollama.com).

## Tính năng

| Tab | Chức năng |
|---|---|
| **Tiến trình** | Theo dõi CPU/RAM theo thời gian thực, dừng tiến trình, tự gắn cờ tiến trình đáng ngờ (chạy từ Temp/Downloads, mạo danh `svchost.exe`, đuôi file kép, file chương trình đã bị xóa), nhờ AI đánh giá mức độ nguy hiểm |
| **Dọn rác** | Dọn file tạm, cache trình duyệt, cache shader, crash dump, file tải về của Windows Update. Thống kê thư mục lớn trong AppData và tìm đồ còn sót của app đã gỡ |
| **Thống kê file** | Dung lượng từng ổ, cây thư mục theo dung lượng, 50 file lớn nhất kèm đánh giá "có xóa được không", thống kê theo loại file, tìm file trùng (so sánh từng byte nội dung) |
| **Gỡ app** | Danh sách app đã cài (đọc từ registry), gỡ bằng trình gỡ gốc của app, tìm và dọn đồ còn sót, nhờ AI tư vấn nên giữ hay gỡ |
| **Cài đặt** | Ngưỡng cảnh báo CPU/RAM, thời gian phải vượt ngưỡng liên tục, khoảng cách giữa hai lần báo, tên model Ollama |

Khi bấm X, app ẩn xuống **khay hệ thống** và tiếp tục cảnh báo CPU/RAM bằng thông báo của Windows. Muốn tắt hẳn thì bấm chuột phải vào icon → "Thoát hẳn".

## Nguyên tắc an toàn

- **Code quyết định, AI chỉ gợi ý.** AI luôn phải trả lời theo JSON schema; trả lời sai định dạng thì bị bỏ. Quy tắc của code có quyền ghi đè AI, ví dụ: tiến trình mạo danh file hệ thống luôn ở mức "Cao"; runtime/driver luôn "Nên giữ"; file trong `C:\Windows` luôn "Không được xóa".
- **Không bao giờ tự xóa hay tự dừng.** Mọi thao tác đều phải bấm xác nhận 2 lần.
- **Ưu tiên xóa vào Thùng rác.** File lớn, file trùng và đồ còn sót đều đi vào Thùng rác nên khôi phục được. Riêng cache và file tạm bị xóa hẳn để thật sự giải phóng ổ đĩa.
- **Không đi theo symlink/junction**, chặn đường dẫn quá ngắn như `C:\`. Backend luôn kiểm tra lại quy tắc, không tin dữ liệu do giao diện gửi lên.
- **Đồ còn sót** chỉ được xóa khi: không khớp app nào đang cài, không có chương trình nào chạy từ đó, không phải thư mục của Windows, và (với thư mục AppData) không được sửa trong 30 ngày.

## AI (tùy chọn)

1. Cài [Ollama](https://ollama.com) rồi tải một model, ví dụ: `ollama pull gemma4:e4b`
2. Nhập tên model trong tab **Cài đặt**.

App ép model chạy hoàn toàn trên GPU (`num_gpu: 999`) và trả lại VRAM sau 2 phút không dùng. Dữ liệu không rời khỏi máy.

## Chạy từ mã nguồn

Cần có: [Rust](https://rustup.rs), [Node.js](https://nodejs.org) 20+, Visual Studio Build Tools (C++).

```bash
npm install
npm run tauri dev      # chạy thử
npm run tauri build    # tạo bộ cài trong src-tauri/target/release/bundle
cd src-tauri && cargo test   # chạy test
```

Muốn dọn hết rác của Windows (`C:\Windows\Temp`, Windows Update) thì chạy app bằng quyền Administrator.

## Cấu trúc

```
src/                    Giao diện React
  features/<tính năng>/ Mỗi tab một thư mục: api.ts (gọi Rust) + các component nhỏ
  components/, lib/     Component và hàm định dạng dùng chung
src-tauri/src/          Backend Rust, mỗi module một việc
  processes, suspicious, process_ai   Tiến trình + gắn cờ + AI
  alerts, settings, tray              Cảnh báo, cài đặt, khay hệ thống
  cleaner, appdata, orphans           Dọn rác, AppData, đồ còn sót
  disk, file_rules, dupes, file_ai    Thống kê file, quy tắc xóa, file trùng
  apps, uninstall, leftovers, app_ai  Gỡ app
  ai, fsutil                          Client Ollama, duyệt file an toàn
```
