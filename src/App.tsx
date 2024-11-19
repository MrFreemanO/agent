import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import './App.css';

function App() {
  const [containerId, setContainerId] = useState<string>('');
  const [status, setStatus] = useState<string>('stopped');
  const [loading, setLoading] = useState<boolean>(false);
  const [error, setError] = useState<string>('');
  const [logs, setLogs] = useState<string>('');

  // 获取容器日志
  const fetchLogs = async () => {
    try {
      const containerLogs = await invoke('get_container_logs');
      setLogs(containerLogs as string);
    } catch (error) {
      console.error('Failed to fetch logs:', error);
      setLogs(`Error fetching logs: ${error}`);
    }
  };

  // 启动容器
  const startContainer = async () => {
    try {
      setLoading(true);
      setError('');
      console.log('Starting container...');

      const id = await invoke('start_container');
      console.log('Container started with ID:', id);

      setContainerId(id as string);
      setStatus('running');

      // 获取初始日志
      await fetchLogs();

      // 等待noVNC服务启动
      setTimeout(async () => {
        const iframe = document.getElementById('vnc-iframe') as HTMLIFrameElement;
        if (iframe) {
          console.log('Connecting to noVNC...');
          iframe.src = 'http://localhost:6070/vnc.html?autoconnect=true&resize=scale';

          // 再次获取日志以查看启动过程
          await fetchLogs();
        }
      }, 2000);
    } catch (error) {
      console.error('Failed to start container:', error);
      setError(`Failed to start container: ${error}`);
      setStatus('error');
    } finally {
      setLoading(false);
    }
  };

  // 停止容器
  const stopContainer = async () => {
    try {
      setLoading(true);
      await invoke('stop_container');
      setContainerId('');
      setStatus('stopped');
      const iframe = document.getElementById('vnc-iframe') as HTMLIFrameElement;
      if (iframe) {
        iframe.src = 'about:blank';
      }
    } catch (error) {
      console.error('Failed to stop container:', error);
    } finally {
      setLoading(false);
    }
  };

  // 重启容器
  const restartContainer = async () => {
    try {
      setLoading(true);
      await invoke('restart_container');
      // 等待noVNC服务重启
      setTimeout(() => {
        const iframe = document.getElementById('vnc-iframe') as HTMLIFrameElement;
        if (iframe) {
          iframe.src = 'http://localhost:6070/vnc.html?autoconnect=true&resize=scale';
        }
      }, 2000);
    } catch (error) {
      console.error('Failed to restart container:', error);
    } finally {
      setLoading(false);
    }
  };

  // 定期更新日志
  useEffect(() => {
    let interval: number;
    if (status === 'running') {
      interval = window.setInterval(fetchLogs, 5000);
    }
    return () => {
      if (interval) {
        clearInterval(interval);
      }
    };
  }, [status]);

  return (
    <div className="container">
      <h1>ConsoleY</h1>
      <div className="controls">
        <button
          onClick={startContainer}
          disabled={loading || status === 'running'}
        >
          {loading ? 'Starting...' : 'Start'}
        </button>
        <button
          onClick={stopContainer}
          disabled={loading || status === 'stopped'}
        >
          Stop
        </button>
        <button
          onClick={restartContainer}
          disabled={loading || status === 'stopped'}
        >
          Restart
        </button>
        <span className={`status ${status}`}>
          Status: {status}
        </span>
      </div>

      {error && (
        <div className="error-message">
          {error}
        </div>
      )}

      <div className="logs-container">
        <h3>Container Logs:</h3>
        <pre>{logs}</pre>
      </div>

      <div className="vnc-container">
        {status === 'running' ? (
          <iframe
            id="vnc-iframe"
            title="ConsoleY Remote Desktop"
            src="about:blank"
            style={{
              width: '100%',
              height: '600px',
              border: 'none',
            }}
          />
        ) : (
          <div className="placeholder">
            Remote desktop will appear here when started
          </div>
        )}
      </div>
    </div>
  );
}

export default App;
