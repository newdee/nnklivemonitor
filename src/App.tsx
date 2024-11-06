import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Button, Input, Form, FloatButton, Table, Tag, Flex, Divider } from "antd";
import { ReloadOutlined } from "@ant-design/icons";
import "./App.css";

interface LiveUser {
  id: number,
  name: string,
  url: string,
  hook: string,
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
  const [waitPageTime, setWaitPageTime] = useState<number>(2000);

  async function addLiveUser() {
    // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
    const newUser: string = await invoke("add_user", { name, url, hook });
    setAddUser(newUser);
    setLiveUser(await invoke("get_all_user", {}));
  }

  const handleFreshPage = async () => {
    setLiveUser(await invoke("get_all_user", {}));
  }

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

    setTimeout(() => { console.log("waiting for loading") }, waitPageTime);

  };


  useEffect(() => {
    const analysisHandle = async () => {
      if (monitorOptions === "停止监控" && currentUser) {
        handleOpenNewWindow(currentUser.url);
        setAnalysisId(await invoke("analysis", {}));
      }
    }
    analysisHandle();

  }, [currentUser]);


  const handleMonitor = async () => {
    if (monitorOptions == "开始监控") {
      await getCurrentInfo();
      setMonitorOptions("停止监控");
    } else if (monitorOptions == "停止监控") {
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
        <Tag color="red">直播地址: {currentUser?.url} </Tag>
      </Flex>
      <FloatButton onClick={handleFreshPage} icon={<ReloadOutlined />} style={{ insetInlineEnd: 24, bottom: 20 }} tooltip={<div>Refresh</div>} />

      <Table dataSource={liveUser} columns={[{ title: "直播商家", dataIndex: "name", key: "name" }, { title: "直播地址", dataIndex: "url", key: "url" }]} pagination={{ position: ["bottomLeft"], showQuickJumper: true, pageSize: 5, }} />

    </main >
  );
}

export default App;
