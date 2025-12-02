# os1000-rs-net

RISC-V 32bit OS (Rust)

## ファイル構造

```text
├── src/                   # カーネル
│   ├── kernel.rs          # エントリーポイント
│   ├── kernel.ld          # リンカスクリプト
│   ├── memory.rs          # メモリ管理
│   ├── process.rs         # プロセス管理
│   ├── fs.rs              # ファイルシステム (tar/ustar)
│   ├── virtio.rs          # Virtioブロックデバイス
│   └── sbi.rs             # SBI
├── common/                # カーネル・ユーザーランド共通
│   └── src/lib.rs         # TrapFrame、システムコール定義等
├── user/                  # ユーザーランド
│   └── src/
│       ├── user.rs        # エントリーポイント
│       ├── user.ld        # リンカスクリプト
│       └── shell.rs       # シェル
├── run.sh                 # ビルド・実行スクリプト
└── opensbi-riscv32-generic-fw_dynamic.bin
```

## 実行

```bash
rustup target add riscv32i-unknown-none-elf
./run.sh
```
