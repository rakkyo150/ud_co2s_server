# ud_co2s_server
UDCO2Sをラズパイに接続したときに、CO2濃度と温度と湿度を返すローカルのAPIサーバーを立てるプログラムです。  
[ud_co2s_viewer](https://github.com/rakkyo150/ud_co2s_viewer)を使えば、ラズパイ以外のPCからCO2濃度を確認することもできます。

## 使い方
まずは、rustを実行できる環境を用意してください。  
[ここ](https://www.rust-lang.org/tools/install)を参考にしてください。  
次に、`example.env`と同じように、`.env`ファイルを作成してください。  
**v1.2.0から、`.env`で設定する変数名の先頭に必ず`VITE_`が必要になったので注意してください。**  
その後、`npm run prod`でサーバーが立ち上がるか確認してください。  
ラズパイの起動時にも自動でサーバーが立ち上がるようにするためには、systemctlのサービスに登録しましょう。
```bash
sudo vim /etc/systemd/system/ud_co2s_server.service
```
サービスファイルの中身は例えば以下のようにしてください。  
ただし、`cargo run --release`を使えるのは一度`npm run build`か`npm run prod`を実行していることが必要です。  
`cargo run --release`の代わりに`npm run prod`でも可。
```service
[Unit]
Description=Set up a server for ud_co2s_server
After=network-online.target
Wants=network-online.target

[Service]
Environment=HOME=/home/pi 
ExecStart=/bin/bash -c 'source $HOME/.bashrc && source $HOME/.profile && cargo run --release'
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

## グラフの表示
v1.2.0から、https://192.168.xxx.xxx:xxxx/graph にアクセスすれば、[ud_co2s_viewer](https://github.com/rakkyo150/ud_co2s_viewer)と同じCO2濃度のグラフが表示されるようになりました。  
ただし、複数台からアクセスすると、ud_co2sの処理能力が間に合わない場合があるので注意です。  
(TODO：一度取得した値は、時間データと共にメモリ上に保存しておいて、一定時間内にリクエストがあった場合は、ud_co2sに再度アクセスせずにメモリ上の値を返すようにすれば解決可能なはず。)
