import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Button, Input, Form, FloatButton, Table, Tag, Flex } from "antd";
import { ReloadOutlined } from "@ant-design/icons";
import "./App.css";

interface LiveUser {
  id: number,
  name: string,
  url: string,
  hook: string,
  status: boolean,
  created_at: string,
  updated_at: string,
}
function App() {
  const [addUser, setAddUser] = useState("");
  const [name, setName] = useState("");
  const [url, setUrl] = useState("");
  const [hook, setHook] = useState("");
  const [liveUser, setLiveUser] = useState<LiveUser[]>([]);
  const [monitorOptions, setMonitorOptions] = useState("开始监控");
  const [analysisId, setAnalysisId] = useState(-1);
  const [currentUser, setCurrentUser] = useState<LiveUser | null>(null);
  const [newWindow, setNewWindow] = useState<WindowProxy | null>(null);
  const waitPageTime = 5000;
  //const [waitPageTime, setWaitPageTime] = useState<number>(5000);

  const disableClickEvents = (event: Event) => {
    event.preventDefault();  // 禁用默认行为
    event.stopPropagation(); // 阻止事件冒泡
  };
  useEffect(() => {
    // 禁用点击事件
    document.addEventListener('contextmenu', disableClickEvents);

    // 清理函数，移除事件监听器
    return () => {
      document.removeEventListener('contextmenu', disableClickEvents);
    };
  }, []);


  async function addLiveUser() {
    // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
    const newUser: string = await invoke("add_user", { name, url, hook });
    setAddUser(newUser);
    setLiveUser(await invoke("get_all_user", {}));
  }

  const handleFreshPage = async () => {
    setLiveUser(await invoke("get_all_user", {}));
  }

  handleFreshPage();
  useEffect(() => {
    const interval = setInterval(() => {
      handleFreshPage();
    }, 10000);

    // 清理定时器，防止内存泄漏
    return () => clearInterval(interval);
  }, []);

  const getCurrentInfo = async () => {
    setCurrentUser(await invoke("get_next_user", {}));
    console.log("current id:", currentUser?.id);
  }

  useEffect(() => {
    getCurrentInfo();
  }, [analysisId]);

  const handleOpenNewWindow = (url: string) => {
    if (!newWindow || newWindow.closed) {
      const windowFeatures = `width=800, height=800, left=0, top=0 `;
      const windowObject = window.open(url, "_blank", windowFeatures);
      setNewWindow(windowObject);  // 保存窗口对象，以便后续操作
    } else {
      newWindow.location.href = url;
    }


  };


  useEffect(() => {
    const analysisHandle = async () => {
      if (monitorOptions === "停止监控" && currentUser) {
        handleOpenNewWindow(currentUser.url);
        setTimeout(async () => { setAnalysisId(await invoke("analysis", {})); }, waitPageTime);
        //setAnalysisId(await invoke("analysis", {}));
      }
    }
    analysisHandle();

  }, [currentUser]);


  const handleMonitor = async () => {
    if (monitorOptions == "开始监控") {
      await getCurrentInfo();
      setMonitorOptions("停止监控");
    } else if (monitorOptions == "停止监控") {
      if (newWindow) {
        newWindow.close();
      }
      setMonitorOptions("开始监控");
    }
  }

  return (
    <main className="container">
      <h1>NNK Live Monitor</h1>
      <Form
        className="row"
        style={{ gap: "2px" }}
        onFinish={addLiveUser}
      >
        <Input
          id="name-input"
          required={true}
          onChange={(e) => setName(e.currentTarget.value)}
          placeholder="商家名"
        />
        <Input
          id="url-input"
          required={true}
          type={"url"}
          onChange={(e) => setUrl(e.currentTarget.value)}
          placeholder="直播地址"
        />
        <Input
          id="hook-input"
          required={true}
          type={"url"}
          onChange={(e) => setHook(e.currentTarget.value)}
          placeholder="上报地址"
        />
        <Button type="primary" htmlType="submit">添加</Button>
        <Button type="primary" onClick={handleMonitor}>{monitorOptions}</Button>
      </Form>
      <p>{addUser}</p>

      <Flex gap="middle" justify="left" align="center" wrap>
        <Tag color="volcano">当前监控 </Tag>
        <Tag color="gold">ID: {currentUser?.id} </Tag>
        <Tag color="magenta">商家: {currentUser?.name} </Tag>
        < a href={currentUser?.url} target="_blank"> <Tag color="red">直播地址: {currentUser?.url} </Tag></a>
      </Flex>
      <FloatButton onClick={handleFreshPage} icon={<ReloadOutlined />} style={{ insetInlineEnd: 24, bottom: 20 }} tooltip={<div>Refresh</div>} />

      <Table dataSource={liveUser} columns={[{ title: "直播商家", dataIndex: "name", key: "name", render: (text) => <Tag color="pink" >{text}</Tag> }, { title: "直播地址", dataIndex: "url", key: "url", render: (text) => <a href={text} target="_blank"><Tag>{text}</Tag></a> }, { title: "更新时间", dataIndex: "updated_at", key: "updated_at", render: (text) => <Tag color="purple">{text}</Tag> }, { title: "直播状态", dataIndex: "status", key: "status", render: (status) => <Tag color={status ? "green" : "red"} > {status ? "正常" : "异常"} </Tag> }]} pagination={{ position: ["bottomLeft"], showQuickJumper: true, pageSize: 5, }} />

    </main >
  );
}

export default App;
