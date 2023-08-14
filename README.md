# ud_co2s_server
UDCO2Sをラズパイに接続したときに、CO2濃度と温度と湿度を返すローカルのAPIサーバーを立てるプログラムです。  
[ud_co2s_viewer](https://github.com/rakkyo150/ud_co2s_viewer)を使えば、ラズパイ以外のPCからCO2濃度を確認することもできます。

## 使い方
まずは、rustを実行できる環境を用意してください。  
[ここ](https://www.rust-lang.org/tools/install)を参考にしてください。  
次に、`example.env`と同じように、`.env`ファイルを作成してください。  
その後、`cargo run`でサーバーが立ち上がるか確認してください。  
サーバーが立ち上がるのを確認出来たら、`cargo build --release`でリリースビルドを作成してください。  
リリースビルドは`./target/release/ud_co2s_server`にあります。  
ラズパイの起動時にも自動でサーバーが立ち上がるようにするためには、systemctlのサービスに登録しましょう。
```bash
sudo vim /etc/systemd/system/ud_co2s_server.service
```
サービスファイルの中身は例えば以下のようにしてください。
```systemd
[Unit]
Description=Set up a server for ud_co2s_server
After=network.target

[Service]
ExecStart=/home/pi/ud_co2s_server/target/release/ud_co2s_server
WorkingDirectory=/home/pi/ud_co2s_server

[Install]
WantedBy=multi-user.target
```
サービスファイルを作成したら、以下のコマンドを実行してください。
```bash
sudo systemctl daemon-reload
sudo systemctl enable ud_co2s_server
sudo systemctl start ud_co2s_server
```

## API
|Method|URI|Models|
|:---|:---|:---|
|GET|https://192.168.xxx.xxx:xxxx/all|{"time":int,"status":{"co2ppm":int,"humidity":float,"temperature":float}}|
|GET|https://192.168.xxx.xxx:xxxx/co2|int|
|GET|https://192.168.xxx.xxx:xxxx/tmp|float|
|GET|https://192.168.xxx.xxx:xxxx/hum|float|
