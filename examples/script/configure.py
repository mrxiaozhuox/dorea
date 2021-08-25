#
# Dorea 测试脚本系统
#
# 系统配置[Web Service]
#

HOSTNAME = "127.0.0.1"
PORT = 3451

GROUP = "default"
PASSWORD = "DOREA@TEST"

META_URL  = "http://" + HOSTNAME + ":" + str(PORT)
GROUP_URL = "http://" + HOSTNAME + ":" + str(PORT) + "/@" + GROUP

import requests


def apply_token() -> str :
    result = requests.post(META_URL + "/auth", data={"password": PASSWORD})
    return result.json()["data"]["token"]
