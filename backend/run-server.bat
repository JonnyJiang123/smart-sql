@echo off
set RUST_LOG=info
set OPENAI_API_KEY=sk-efogltfmnyievufkpiesgtdwzjurlsiqdneotlkwufambref
set OPENAI_MODEL=Qwen/QwQ-32B
set OPENAI_API_BASE_URL=https://api.siliconflow.cn/v1

echo 启动智能SQLer后端服务...
.\target\debug\smart-sql-backend.exe
pause
