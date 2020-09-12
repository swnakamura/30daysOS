if __name__ == '__main__':
    with open("./hankaku.txt", 'r') as f, open("../../build/font.in", 'w') as f_out:
        f_out.writelines("[\n")
        for item in f.readlines():
            item = item.strip()
            if item.startswith("char"):
                f_out.writelines("[")
            elif item == "":
                f_out.writelines("],\n")
            else:
                val = 0
                t = 128
                for char in item:
                    if char == "*":
                        val += t
                    t //= 2
                f_out.write("0x{:x},".format(val))
        f_out.writelines("]")
