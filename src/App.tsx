import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import * as Switch from "@radix-ui/react-switch";
// import * as Select from "@radix-ui/react-select";
// import { CheckIcon, ChevronDownIcon } from "@radix-ui/react-icons";

import './App.css';

function App() {
  // 注释掉未使用的状态变量
  // const [containerId, setContainerId] = useState<string>('');
  const [status, setStatus] = useState<string>('stopped');

  const [loading, setLoading] = useState<boolean>(false);

  // const [isSide, setIsSide] = useState(false);

  // const [active, setActive] = useState(1);

  const [checked, setChecked] = useState(true);

  // const [value, setValue] = useState('');

  // 启动容器
  const startContainer = async () => {
    try {
      setLoading(true);
      await invoke('start_container');
      // setContainerId(id as string);  // 注释掉未使用的 setter
      setStatus('running');

      // 等待noVNC服务启动
      setTimeout(async () => {
        const iframe = document.getElementById('vnc-iframe') as HTMLIFrameElement;
        if (iframe) {
          iframe.src = 'http://localhost:6070/vnc.html?autoconnect=true&resize=scale';
        }
      }, 2000);

    } catch (error) {
      setStatus('error');
    } finally {
      setLoading(false);
    }
  };

  // 停止容器
  // const stopContainer = async () => {
  //   try {
  //     setLoading(true);
  //     await invoke('stop_container');
  //     setContainerId('');
  //     setStatus('stopped');
  //     const iframe = document.getElementById('vnc-iframe') as HTMLIFrameElement;
  //     if (iframe) {
  //       iframe.src = 'about:blank';
  //     }
  //   } catch (error) {
  //     console.error('Failed to stop container:', error);
  //   } finally {
  //     setLoading(false);
  //   }
  // };

  // 重启容器
  // const restartContainer = async () => {
  //   try {
  //     setLoading(true);
  //     await invoke('restart_container');
  //     // 等待noVNC服务重启
  //     setTimeout(() => {
  //       const iframe = document.getElementById('vnc-iframe') as HTMLIFrameElement;
  //       if (iframe) {
  //         iframe.src = 'http://localhost:6070/vnc.html?autoconnect=true&resize=scale';
  //       }
  //     }, 2000);
  //   } catch (error) {
  //     console.error('Failed to restart container:', error);
  //   } finally {
  //     setLoading(false);
  //   }
  // };

  useEffect(() => {
    startContainer();
  }, [])

  // 允许人类操作界面
  const onCheckedChange = (checked: boolean) => {
    setChecked(checked);
  };

  // 选择分辨率
  // const onResolutionChange = (value: string) => {
  //   setValue(value);
  // }

  return (
    <div className="container">
      <div className='container_content'>
        <div className='title_container'>
          <div>ConsoleY</div>
          <div className='container_option_content_item'>
            <div className='container_option_content_item_label'>允许人类操作界面</div>
            <div className='container_option_content_item_com'>
              <Switch.Root className={checked ? "SwitchRoot" : "SwitchNoRoot"} checked={checked} onCheckedChange={(checked) => onCheckedChange(checked)}>
                <Switch.Thumb className="SwitchThumb" />
              </Switch.Root>
            </div>
          </div>
          {/* <div className='title_container_right' onClick={() => setIsSide(!isSide)}>
            <svg className="icon_arrow" aria-hidden="true" xmlns="http://www.w3.org/2000/svg" width="24" height="24" fill="none" viewBox="0 0 24 24">
              <path stroke="currentColor" strokeLinecap="round" strokeWidth="2" d="M5 7h14M5 12h14M5 17h14" />
            </svg>
          </div> */}
        </div>
        <div className="vnc-container">
          {!checked ? <div className="vnc-container_model" /> : ''}
          {
            loading ?
              <div className="placeholder">Loading...</div> :
              status === 'running' ? (
                <iframe
                  id="vnc-iframe"
                  title="ConsoleY Remote Desktop"
                  src="about:blank"
                  style={{
                    width: '100%',
                    height: '100%',
                    border: 'none',
                  }}
                />
              ) : (
                <div className="placeholder">
                  Remote desktop will appear here when started
                </div>
              )
          }
        </div>
      </div>
      {/* {
        // isSide ? <div className='side' /> : ''
      } */}
      {/* {
        // isSide ? <div className='container_option'>
        //   <div className='container_option_tab'>
        //     <div className={active === 1 ? "container_option_tab_active" : "container_option_tab_item"} onClick={() => setActive(1)}>设置</div>
        //     <div className={active === 2 ? "container_option_tab_active" : "container_option_tab_item"} onClick={() => setActive(2)}>调试</div>
        //     <div className={active === 3 ? "container_option_tab_active" : "container_option_tab_item"} onClick={() => setActive(3)}>应用程序</div>
        //   </div>
        //   <div className='container_option_content'>
        //     {active === 1 ?
        //       <div className='container_option_content_box'>
        //         <div className='container_option_content_item'>
        //           <div className='container_option_content_item_label'>允许人类操作界面</div>
        //           <div className='container_option_content_item_com'>
        //             <Switch.Root className={checked ? "SwitchRoot" : "SwitchNoRoot"} checked={checked} onCheckedChange={(checked) => onCheckedChange(checked)}>
        //               <Switch.Thumb className="SwitchThumb" />
        //             </Switch.Root>
        //           </div>
        //         </div>
        //         <div className='container_option_content_item'>
        //           <div className='container_option_content_item_label'>桌面分辨率</div>
        //           <div className='container_option_content_item_com'>
        //             <Select.Root value={value} onValueChange={(value) => onResolutionChange(value)}>
        //               <Select.Trigger className="SelectTrigger">
        //                 <Select.Value placeholder="Select a resolution…" />
        //                 <Select.Icon className="SelectIcon">
        //                   <ChevronDownIcon />
        //                 </Select.Icon>
        //               </Select.Trigger>
        //               <Select.Portal>
        //                 <Select.Content className="SelectContent">
        //                   <Select.Viewport className="SelectViewport">
        //                     <Select.Group>
        //                       <SelectItem value="1024*768">1024*768</SelectItem>
        //                       <SelectItem value="1920*1080">1920*1080</SelectItem>
        //                       <SelectItem value="2560*1440">2560*1440</SelectItem>
        //                       <SelectItem value="3840*2160">3840*2160</SelectItem>
        //                     </Select.Group>
        //                   </Select.Viewport>
        //                 </Select.Content>
        //               </Select.Portal>
        //             </Select.Root>
        //           </div>
        //         </div>
        //       </div>
        //       : ''}
        //     {active === 2 ? <div className='container_option_content_box'></div> : ''}
        //     {active === 3 ? <div className='container_option_content_box'></div> : ''}
        //   </div>
        // </div> : ''
      } */}
    </div>
  );
}

// const SelectItem = React.forwardRef(
//   ({ children, ...props }: { children: React.ReactNode, value: string }, forwardedRef) => {
//     return (
//       <Select.Item
//         {...props}
//         ref={forwardedRef as any}
//         className="SelectItem"
//       >
//         <Select.ItemText>{children}</Select.ItemText>
//         <Select.ItemIndicator className="SelectItemIndicator">
//           <CheckIcon />
//         </Select.ItemIndicator>
//       </Select.Item>
//     );
//   },
// );

export default App;
