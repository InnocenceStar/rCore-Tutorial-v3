import os

# 修改linker.ld配置，为每个APP指定不同的入口地址；
# 内核需要知道这些APP的入口地址，因为现在还不支持动态加载；

base_address = 0x80400000
step = 0x20000
linker = "src/linker.ld"

app_id = 0
apps = os.listdir("src/bin")
apps.sort()
for app in apps:
    app = app[: app.find(".")]
    lines = []
    lines_before = []
    # 搜索文件中所有的地址并替换
    with open(linker, "r") as f:
        for line in f.readlines():
            lines_before.append(line)
            line = line.replace(hex(base_address), hex(base_address + step * app_id))
            lines.append(line)
    with open(linker, "w+") as f:
        f.writelines(lines)
    os.system("cargo build --bin %s --release" % app)
    print(
        "[build.py] application %s start with address %s"
        % (app, hex(base_address + step * app_id))
    )
    with open(linker, "w+") as f:
        f.writelines(lines_before)
    app_id = app_id + 1
