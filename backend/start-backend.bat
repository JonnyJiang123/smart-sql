@echo off
cd /d %~dp0
set OPENAI_API_KEY=sk-efogltfmnyievufkpiesgtdwzjurlsiqdneotlkwufambref
set OPENAI_MODEL=Qwen/QwQ-32B
set OPENAI_API_BASE_URL=https://api.siliconflow.cn/v1
set RUST_LOG=info
set RUST_BACKTRACE=full
.\target\debug\smart-sql-backend.exe
