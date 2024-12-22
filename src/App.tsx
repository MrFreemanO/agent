import { useEffect, useState } from 'react';
import * as Switch from "@radix-ui/react-switch";

import './App.css';

function App() {
  const [status, setStatus] = useState<string>('stopped');
  const [loading, setLoading] = useState<boolean>(false);
  const [checked, setChecked] = useState(false);
  const [retryCount, setRetryCount] = useState(0);
  const maxRetries = 30;

  // Check if port is accessible
  const checkPort = async (port: number): Promise<boolean> => {
    console.log(`Checking VNC port ${port}...`);
    try {
      // Create WebSocket connection to VNC port
      const ws = new WebSocket(`ws://localhost:${port}`);
      
      return new Promise((resolve) => {
        ws.onopen = () => {
          console.log(`Successfully connected to VNC port ${port}`);
          ws.close();
          resolve(true);
        };
        
        ws.onerror = (error) => {
          console.error(`Failed to connect to VNC port ${port}:`, error);
          resolve(false);
        };
        
        // Set connection timeout
        setTimeout(() => {
          console.log(`Connection to port ${port} timed out`);
          ws.close();
          resolve(false);
        }, 2000);
      });
    } catch (error) {
      console.error(`Error checking port ${port}:`, error);
      return false;
    }
  };

  const checkAndRefreshVnc = async () => {
    console.log(`Checking VNC service (attempt ${retryCount + 1}/${maxRetries})...`);
    
    try {
      // Check VNC port (5800)
      const isVNCReady = await checkPort(5800);
      
      if (isVNCReady) {
        console.log('VNC service is ready, updating status...');
        setStatus('running');
        setLoading(false);
        setRetryCount(0);
      } else {
        console.log('VNC service not ready yet, retrying...');
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

  // Check service status periodically
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

  return (
    <div className="container">
      <div className='container_content'>
        <div className='title_container'>
          <div>
            ConsoleY
          </div>
          <div className='container_option_content_item'>
            <div className='container_option_content_item_label'>Allow human operation</div>
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
              <div className="loading-spinner" />
              <div>
                {retryCount > 0 ? 
                  `Connecting to remote desktop... (${retryCount}/${maxRetries})` : 
                  'Connecting to remote desktop...'
                }
              </div>
              <div className="status-text">Status: {status}</div>
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
              {status === 'error' ? (
                <>
                  <svg xmlns="http://www.w3.org/2000/svg" width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                    <circle cx="12" cy="12" r="10"/>
                    <line x1="12" y1="8" x2="12" y2="12"/>
                    <line x1="12" y1="16" x2="12" y2="16"/>
                  </svg>
                  <div>Remote desktop connection failed</div>
                </>
              ) : (
                <>
                  <div className="loading-placeholder">
                    <div className="loading-spinner-container">
                      <span className="load_animation"></span>
                    </div>
                  </div>
                  {/* <div>Loading desktop...</div> */}
                </>
              )}
            </div>
          )}
        </div>
      </div>
    </div>
  );
}

export default App;
