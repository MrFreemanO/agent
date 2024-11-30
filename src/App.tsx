import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import * as Switch from "@radix-ui/react-switch";

import './App.css';

function App() {
  const [status, setStatus] = useState<string>('stopped');
  const [loading, setLoading] = useState<boolean>(false);
  const [checked, setChecked] = useState(true);
  const [retryCount, setRetryCount] = useState(0);
  const maxRetries = 30;

  // 检查端口是否可访问
  const checkPort = async (port: number): Promise<boolean> => {
    try {
      const response = await fetch(`http://localhost:${port}/vnc.html`, {
        method: 'HEAD',
        mode: 'no-cors'
      });
      console.log(`Port ${port} check response:`, response.type);
      return true;
    } catch (error) {
      console.log(`Port ${port} not accessible:`, error);
      return false;
    }
  };

  const checkAndRefreshVnc = async () => {
    console.log(`Checking VNC service (attempt ${retryCount + 1}/${maxRetries})...`);
    
    try {
      const isNoVNCReady = await checkPort(6070);
      
      if (isNoVNCReady) {
        console.log('noVNC service is ready, updating status...');
        setStatus('running');
        setLoading(false);
      } else {
        console.log('noVNC service not ready yet, retrying...');
        retryConnection();
      }
    } catch (error) {
      console.error('Error checking VNC service:', error);
      retryConnection();
    }
  };

  const retryConnection = () => {
    if (retryCount < maxRetries) {
      setRetryCount(prev => prev + 1);
      setTimeout(checkAndRefreshVnc, 2000);
    } else {
      console.error('Failed to connect to VNC after maximum retries');
      setStatus('error');
      setLoading(false);
    }
  };

  const startContainer = async () => {
    try {
      setLoading(true);
      setRetryCount(0);
      console.log("Starting container...");
      
      await invoke('start_container');
      console.log("Container started successfully");
      
    } catch (error) {
      console.error('Container start error:', error);
      setStatus('error');
      setLoading(false);
    }
  };

  // 监听后端事件
  useEffect(() => {
    const unlisten = listen('vnc-ready', () => {
      console.log('Received vnc-ready event from backend');
      checkAndRefreshVnc();
    });

    return () => {
      unlisten.then(fn => fn());
    };
  }, []);

  // 定期检查服务状态
  useEffect(() => {
    let intervalId: number;

    if (status !== 'running' && !loading) {
      intervalId = window.setInterval(() => {
        checkAndRefreshVnc();
      }, 5000);
    }

    return () => {
      if (intervalId) {
        clearInterval(intervalId);
      }
    };
  }, [status, loading]);

  // 组件挂载时自动启动容器
  useEffect(() => {
    startContainer();
  }, []);

  return (
    <div className="container">
      <div className='container_content'>
        <div className='title_container'>
          <div>ConsoleY</div>
          <div className='container_option_content_item'>
            <div className='container_option_content_item_label'>允许人类操作界面</div>
            <div className='container_option_content_item_com'>
              <Switch.Root 
                className={checked ? "SwitchRoot" : "SwitchNoRoot"} 
                checked={checked} 
                onCheckedChange={setChecked}
              >
                <Switch.Thumb className="SwitchThumb" />
              </Switch.Root>
            </div>
          </div>
        </div>
        <div className="vnc-container">
          {!checked && <div className="vnc-container_model" />}
          {loading ? (
            <div className="placeholder">
              Loading... {retryCount > 0 && `(Attempt ${retryCount}/${maxRetries})`}
              <div>Status: {status}</div>
            </div>
          ) : status === 'running' ? (
            <iframe
              id="vnc-iframe"
              title="ConsoleY Remote Desktop"
              src={`http://localhost:6070/vnc.html?autoconnect=true&resize=scale&password=&t=${new Date().getTime()}`}
              style={{
                width: '100%',
                height: '100%',
                border: 'none',
              }}
            />
          ) : (
            <div className="placeholder">
              {status === 'error' ? 
                'Failed to connect to remote desktop' : 
                'Remote desktop will appear here when started'}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}

export default App;
