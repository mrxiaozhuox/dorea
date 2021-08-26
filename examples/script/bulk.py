#
# Dorea 测试脚本系统
#
# 批量写入 & 加载 [Web Service]
#

import sys, math
import threading
import configure
import requests

# 调用 configure 中的申请函数尝试获取 JWT Token 值
jwt_token = configure.apply_token()

if jwt_token == "":
    print("JWT 验证获取异常！")
    sys.exit(0)

# 使用 JWT 尝试获取信息：/@default/INFO - [POST] 
result = requests.post(configure.GROUP_URL + "/INFORMATION",headers={"Authorization": "Bearer " + jwt_token});

if result.status_code != 200:
    print("接口请求异常！")
    sys.exit(0)

# 批量插入脚本
TOTAL_NUMBER = 1000 * 10
for i in range(TOTAL_NUMBER):
    temp = requests.post(
        configure.GROUP_URL + "/SET",
        headers={ "Authorization": "Bearer " + jwt_token },
        data={ "key": "D@" + str(i), "value": "hello world" }
    )
    print("Key: " + "D@" + str(i) + " | " + temp.text)
print("运行结束！")