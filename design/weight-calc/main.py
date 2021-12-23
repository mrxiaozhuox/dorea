def main():

    max_index_number = 2048

    while True:

        cmd = input("~> ")
        if cmd[0:4] == "set ":
            
            sub_cmd: str = cmd[4:]
            sub_cmd = sub_cmd.split(" ")
            if len(sub_cmd) < 2:
                print("unknown command")
                continue
            
            if sub_cmd[0] == "max_index_number":
                max_index_number = int(sub_cmd[1])
                print("max_index_number = " + sub_cmd[1])
                print("max_group_index_number = " + str(int(max_index_number / 4)))
                continue

        if cmd[0:5] == "calc ":
            sub_cmd: str = cmd[5:]
            sub_cmd = sub_cmd.split(" ")
            if len(sub_cmd) < 2:
                print("unknown command")
                continue
            res = calc(int(sub_cmd[0]), int(sub_cmd[1]), (max_index_number / 4))
            print("result = " + str(res))
            continue

        print("unknown command")
        continue

def calc(weight, index_num, max_group_index_num):
    return int(weight * (max_group_index_num / index_num))

if __name__ == "__main__":
    main()