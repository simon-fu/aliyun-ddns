# aliyun-ddns

## 操作步骤
- 阿里云官网添加 DNS 记录
    - 主机记录 dev.test，记录类型 A，记录值 127.0.0.1  

- 阿里云官网生成 AccessKey
    - RAM 访问控制 -> 用户 -> 创建用户 -> 勾选 OpenAPI 调用访问启用
    - RAM 访问控制 -> 用户 -> 点击用户名 -> 认证管理 -> 创建 AccessKey -> 复制保存 AccessKeyId 和 AccessKeySecret
    - RAM 访问控制 -> 授权 -> 给用户添加权限 AliyunDNSFullAccess  

- 下载 aliyun cli，本地配置 AccessKey， 


    - 运行  
        ```
        $ aliyun configure set \
            --profile akProfile \
            --mode AK \
            --region cn-hangzhou \
            --access-key-id AccessKeyId \
            --access-key-secret AccessKeySecret
        ```

- 测试 aliyun cli
    - 获取域名所有记录， 找到 dev.test 对应 RecordId
        ```
        $ ./aliyun alidns DescribeDomainRecords --region cn-hangzhou --DomainName 'rtcsdk.com'
        ```
    - 修改记录值
        ```
        $ ./aliyun alidns UpdateDomainRecord --region cn-hangzhou --RecordId 831868602766839808 --RR 'dev.test' --Type A --Value '127.0.0.2'
        ```

    - 确认生效
        ```
        $ ping dev.test.rtcsdk.com
        ```

- 运行本程序
    - 运行方式一
        ```
        $ cargo run -- --domain rtcsdk.com --rr dev.test --cli "/Users/simon/simon/myhome/mini/aliyun/aliyun"
        ```
        - 手工取外网地址
            ````
            $ curl jsonip.com
            ````
        - 确认生效
            ```
            $ ping dev.test.rtcsdk.com
            ```

    - 运行方式二
        ```
        $ cargo run -- --domain rtcsdk.com --rr dev.test --cli "/Users/simon/simon/myhome/mini/aliyun/aliyun" --ping "udp://39.105.43.146:5000?line=hello-ddns"
        ```
      - ping 是向一个服务器周期发 udp 包，line是发送内容
      - 在服务器上运行 nc -v -l -p 5000  可得到公网地址，这个命令只有效一次，每次都要重新运行

- 阿里云链接  
    [配置凭证](https://help.aliyun.com/document_detail/110341.html?spm=a2c4g.121259.0.0.56ee4007veBBj5) ， [API 调试](https://next.api.aliyun.com/api/Alidns/2015-01-09/DescribeDomainRecords?tab=CLI&params={%22DomainName%22:%22rtcsdk.com%22}) ， [RAM 访问控制](https://ram.console.aliyun.com/overview) ， [数字证书管理服务](https://yundun.console.aliyun.com/?spm=5176.12818093.ProductAndResource--ali--widget-product-recent.dre1.473c16d06i4fAt&p=cas#/overview/cn-hangzhou) ， [域名解析](https://dns.console.aliyun.com/?spm=5176.12818093.ProductAndResource--ali--widget-product-recent.dre0.3be916d0f4C8EM#/dns/domainList)

## MacOS 开机自运行
- 参考 [怎么在命令行给 macOS 设置开机启动任务](http://davidleee.com/2018/11/07/login-items-for-macos/)  
    ```
    $ sudo vim /Library/LaunchDaemons/dev.test.ddns.service.plist
    ```
    ```
    <?xml version="1.0" encoding="UTF-8"?>
    <!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
    <plist version="1.0">
    <dict>
        <key>Label</key>
        <string>dev.test.ddns.service.plist</string>

        <key>Program</key>
        <string>/Users/simon/simon/aliyun/bin/run-ddns.sh</string>

        <key>UserName</key>
        <string>simon</string>

        <key>Disabled</key>
        <false/>

        <key>RunAtLoad</key>
        <true/>

        <key>StandardOutPath</key>
        <string>/tmp/ddns-startup.stdout</string>

        <key>StandardErrorPath</key>
        <string>/tmp/ddns-startup.stderr</string>

    </dict>
    </plist>
    ```
