
from fastapi import FastAPI, HTTPException
from pydantic import BaseModel
import os
import subprocess

app = FastAPI()

class CommandRequest(BaseModel):
    action: str
    args: list[str] = []

@app.get("/")
def read_root():
    return {"status": "ConsoleX API is running"}

@app.post("/run")
def run_command(cmd: CommandRequest):
    try:
        if cmd.action == "open":
            subprocess.Popen(cmd.args)
            return {"status": "started", "args": cmd.args}
        elif cmd.action == "shell":
            result = subprocess.run(cmd.args, capture_output=True, text=True)
            return {"stdout": result.stdout, "stderr": result.stderr}
        else:
            raise HTTPException(status_code=400, detail="Unknown action")
    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))
