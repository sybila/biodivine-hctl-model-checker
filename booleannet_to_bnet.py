import sys

before = sys.argv[1]
after = sys.argv[2]

with open(before, 'r') as rf:
    wf = open(after, "w")
    lines = rf.read().split("\n")[1:]
    for line in lines:
        line = line.replace("*=", ", ")
        line = line.replace("not", "!")
        line = line.replace("and", "&")
        line = line.replace("or", "|")
        wf.write(line + "\n")
