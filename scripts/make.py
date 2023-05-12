import re
from pathlib import Path
import matplotlib.pyplot as plt

num_pattern = "(\d+\.\d+)\s(.)?s"
pattern = r'Benchmarking (\S+)\nBenchmarking (?:\S+).+(?:\n\nWarning:.+)?\nBenchmarking.+\nBenchmarking.+\n(?:\S+)\s+time:\s+\[\S+ \S+ (\S+ \S+) \S+ \S+\]\n(?:.+\n){0,5}(?:slope.+\n)?mean.+ std\. dev\.\s+\[(\S+ \S+) (\S+ \S+)\]\nmedian.+'

def num(s):
    match = re.match(num_pattern, s).groups()
    n = float(match[0])
    u = match[1]
    if u == None:
        return n * 1000
    elif u == "m":
        return n
    elif u == "µ":
        return n / 1000


data = Path('results.txt').read_text()
match = re.findall(pattern, data)

match = [list(m) for m in match]

for m in match:
    m[1] = num(m[1])
    m[2] = num(m[2])
    m[3] = num(m[3])

def plot_time():
    # Setup time
    pat = r'setup/(\w+)/(\d+)'
    points = [x for x in match if re.match(pat, x[0])]

    for i in range(len(points)):
        m = re.match(pat, points[i][0]).groups()
        points[i] = [m[0]] + [m[1]] + points[i][1:]

    x = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]

    # sequential x, yerrorlow, yerrorhigh
    seq_y = [x[2] for x in points if x[0] == "seq"]
    seq_yel = [x[3] for x in points if x[0] == "seq"]
    seq_yeh = [x[4] for x in points if x[0] == "seq"]
    seq_y_error = [seq_yel, seq_yeh]

    # par x, yerrorlow, yerrorhigh
    par_y = [x[2] for x in points if x[0] == "par"]
    par_yel = [x[3] for x in points if x[0] == "par"]
    par_yeh = [x[4] for x in points if x[0] == "par"]
    par_y_error = [par_yel, par_yeh]

    # disk x, yerrorlow, yerrorhigh
    disk_y = [x[2] for x in points if x[0] == "disk"]
    disk_yel = [x[3] for x in points if x[0] == "disk"]
    disk_yeh = [x[4] for x in points if x[0] == "disk"]
    disk_y_error = [disk_yel, disk_yeh]
    
    fig, ax = plt.subplots()
    plt.xlabel("Number of points (10k)")
    plt.ylabel("Time taken (ms)")
    plt.xticks(x)
    ax.errorbar(x, seq_y, yerr=seq_y_error, label='Sequential', capsize=5)
    ax.errorbar(x, par_y, yerr=par_y_error, label='Parallel (8192)', capsize=5)
    ax.errorbar(x, disk_y, yerr=disk_y_error, label='Disk', capsize=5)
    ax.legend()
    ax.grid()
    fig.savefig("setup_time.png", dpi=300)


plot_time()

def plot_block_size():
    pat = r"block_size/(\d+)"
    points = [x for x in match if re.match(pat, x[0])]
    for i in range(len(points)):
        m = re.match(pat, points[i][0]).groups()
        points[i] = [int(m[0])] + points[i][1:]

    x = [512, 1024, 2048, 4096, 8192, 16384, 32768]
    y = [x[1] for x in points]
    yel = [x[2] for x in points]
    yeh = [x[3] for x in points]
    yerr = [yel, yeh]

    none_line = [[x for x in match if re.match(r"block_size/None", x[0])][0][1]] * len(x)

    fig, ax = plt.subplots()
    plt.xlabel("Block size")
    plt.ylabel("Time taken (ms)")
    plt.xticks(x)
    ax.errorbar(x, y, yerr=yerr, label='Parallel', capsize=10)
    ax.errorbar(x, none_line, label='Control', capsize=10)
    ax.legend()
    ax.grid()
    ax.set_xscale('log', base=2)
    fig.savefig("block_size.png", dpi=300)

plot_block_size()

def plot_cull():
    pat = r"100k_d(\d+)_flat/cull_(\w+)/(.+)"
    points = [x for x in match if re.match(pat, x[0])]
    for i in range(len(points)):    
        m = re.match(pat, points[i][0]).groups()
        # dims, series, cull perecentage
        points[i] = [int(m[0])] + [m[1]] + [m[2]] + points[i][1:]
    
    pat2 = r"100k_d(\d+)_flat/par"
    points2 = [x for x in match if re.match(pat2, x[0])]
    for i in range(len(points2)):
        m = re.match(pat2, points2[i][0]).groups()
        points2[i] = [int(m[0])] + points2[i][1:]


    x = [10, 12, 14, 16, 18, 20, 22, 24, 26, 28, 30]

    fig, ax = plt.subplots()
    plt.xlabel("Dimensions")
    plt.ylabel("Time taken (ms)")
    plt.xticks(x)

    y_control = [x[1] for x in points2]
    y_controlel = [x[2] for x in points2]
    y_controleh = [x[3] for x in points2]
    y_controlerr = [y_controlel, y_controleh]
    ax.errorbar(x, y_control, yerr=y_controlerr, label='Control (no culling)', capsize=5)


    y_pop_95 = [x[3] for x in points if x[1] == "pop" and x[2] == "0.95"]
    y_pop_95el = [x[4] for x in points if x[1] == "pop" and x[2] == "0.95"]
    y_pop_95eh = [x[5] for x in points if x[1] == "pop" and x[2] == "0.95"]
    y_pop_95err = [y_pop_95el, y_pop_95eh]
    ax.errorbar(x, y_pop_95, yerr=y_pop_95err, label='Population (95%)', capsize=5)


    y_sim_95 = [x[3] for x in points if x[1] == "sim" and x[2] == "0.95"]
    y_sim_95el = [x[4] for x in points if x[1] == "sim" and x[2] == "0.95"]
    y_sim_95eh = [x[5] for x in points if x[1] == "sim" and x[2] == "0.95"]
    y_sim_95err = [y_sim_95el, y_sim_95eh]
    ax.errorbar(x, y_sim_95, yerr=y_sim_95err, label='Similarity (95%)', capsize=5)

    y_all_95 = [x[3] for x in points if x[1] == "all" and x[2] == "0.95"]
    y_all_95el = [x[4] for x in points if x[1] == "all" and x[2] == "0.95"]
    y_all_95eh = [x[5] for x in points if x[1] == "all" and x[2] == "0.95"]
    y_all_95err = [y_all_95el, y_all_95eh]
    ax.errorbar(x, y_all_95, yerr=y_all_95err, label='Population + Similarity (95%)', capsize=5)

    ax.legend()
    ax.grid()
    fig.savefig("cull_95.png", dpi=300)

    fig, ax = plt.subplots()
    plt.xlabel("Dimensions")
    plt.ylabel("Time taken (ms)")
    plt.xticks(x)
    ax.errorbar(x, y_control, yerr=y_controlerr, label='Control (no culling)', capsize=5)

    y_pop_80 = [x[3] for x in points if x[1] == "pop" and x[2] == "0.8"]
    y_pop_80el = [x[4] for x in points if x[1] == "pop" and x[2] == "0.8"]
    y_pop_80eh = [x[5] for x in points if x[1] == "pop" and x[2] == "0.8"]
    y_pop_80err = [y_pop_80el, y_pop_80eh]
    ax.errorbar(x, y_pop_80, yerr=y_pop_80err, label='Population (80%)', capsize=5)

    y_sim_80 = [x[3] for x in points if x[1] == "sim" and x[2] == "0.8"]
    y_sim_80el = [x[4] for x in points if x[1] == "sim" and x[2] == "0.8"]
    y_sim_80eh = [x[5] for x in points if x[1] == "sim" and x[2] == "0.8"]
    y_sim_80err = [y_sim_80el, y_sim_80eh]
    ax.errorbar(x, y_sim_80, yerr=y_sim_80err, label='Similarity (80%)', capsize=5)

    y_all_80 = [x[3] for x in points if x[1] == "all" and x[2] == "0.8"]
    y_all_80el = [x[4] for x in points if x[1] == "all" and x[2] == "0.8"]
    y_all_80eh = [x[5] for x in points if x[1] == "all" and x[2] == "0.8"]
    y_all_80err = [y_all_80el, y_all_80eh]
    ax.errorbar(x, y_all_80, yerr=y_all_80err, label='Population + Similarity (80%)', capsize=5)

    ax.legend()
    ax.grid()
    fig.savefig("cull_80.png", dpi=300)
    
    return y_pop_95, y_pop_95err

cull_p95, cull_p95_err = plot_cull()

def plot_lin():
    pat = r"100k_d(\d+)_flat/(lin(?:_par)?)"
    points = [x for x in match if re.match(pat, x[0])]
    for i in range(len(points)):    
        m = re.match(pat, points[i][0]).groups()
        # dim, lin/par
        points[i] = [int(m[0])] + [m[1]] + points[i][1:]

    x = [10, 12, 14, 16, 18, 20, 22, 24, 26, 28, 30]

    fig, ax = plt.subplots()
    plt.xlabel("Dimensions")
    plt.ylabel("Time taken (ms)")
    plt.xticks(x)

    y = [x[2] for x in points if x[1] == "lin"]
    y_el = [x[3] for x in points if x[1] == "lin"]
    y_eh = [x[4] for x in points if x[1] == "lin"]
    y_err = [y_el, y_eh]
    ax.errorbar(x, y, yerr=y_err, label='Linear', capsize=5)

    y_par = [x[2] for x in points if x[1] == "lin_par"]
    y_par_el = [x[3] for x in points if x[1] == "lin_par"]
    y_par_eh = [x[4] for x in points if x[1] == "lin_par"]
    y_par_err = [y_par_el, y_par_eh]
    ax.errorbar(x, y_par, yerr=y_par_err, label='Linear (Parallel)', capsize=5)

    ax.legend()
    ax.grid()
    fig.savefig("lin_par.png", dpi=300)
    return y, y_err, y_par, y_par_err

lin, lin_err, lin_par, lin_par_err = plot_lin()

def plot_seq():
    pat = r"100k_d(\d+)_flat/((?:seq)|(?:par))"
    points = [x for x in match if re.match(pat, x[0])]
    for i in range(len(points)):    
        m = re.match(pat, points[i][0]).groups()
        # dim, seq_par
        points[i] = [int(m[0])] + [m[1]] + points[i][1:]

    x = [10, 12, 14, 16, 18, 20, 22, 24, 26, 28, 30]

    fig, ax = plt.subplots()
    plt.xlabel("Dimensions")
    plt.ylabel("Time taken (ms)")
    plt.xticks(x)

    y = [x[2] for x in points if x[1] == "seq"]
    y_el = [x[3] for x in points if x[1] == "seq"]
    y_eh = [x[4] for x in points if x[1] == "seq"]
    y_err = [y_el, y_eh]
    ax.errorbar(x, y, yerr=y_err, label='Sequential', capsize=5)

    y_par = [x[2] for x in points if x[1] == "par"]
    y_par_el = [x[3] for x in points if x[1] == "par"]
    y_par_eh = [x[4] for x in points if x[1] == "par"]
    y_par_err = [y_par_el, y_par_eh]
    ax.errorbar(x, y_par, yerr=y_par_err, label='Parallel (8192)', capsize=5)

    ax.legend()
    ax.grid()
    fig.savefig("seq_par.png", dpi=300)
    return y, y_err, y_par, y_par_err

seq, seq_err, par, par_err = plot_seq()

def plot_pars():
    pat = r"100k_d(\d+)_flat/disk"
    points = [x for x in match if re.match(pat, x[0])]
    for i in range(len(points)):
        m = re.match(pat, points[i][0]).groups()
        # dim, seq_par
        points[i] = [int(m[0])] + points[i][1:]

    x = [10, 12, 14, 16, 18, 20, 22, 24, 26, 28, 30]

    y = [x[1] for x in points]
    y_el = [x[2] for x in points]
    y_eh = [x[3] for x in points]
    y_err = [y_el, y_eh]

    fig, ax = plt.subplots()
    plt.xlabel("Dimensions")
    plt.ylabel("Time taken (ms)")
    plt.xticks(x)

    ax.errorbar(x, cull_p95, yerr=cull_p95_err, label='Cull (95%, Population)', capsize=5)
    ax.errorbar(x, lin_par, yerr=lin_par_err, label='Linear (Parallel)', capsize=5)
    ax.errorbar(x, par, yerr=par_err, label='Parallel (8192)', capsize=5)
    ax.errorbar(x, y, yerr=y_err, label='Disk', capsize=5)


    ax.legend()
    ax.grid()
    fig.savefig("pars.png", dpi=300)

plot_pars()

def plot_seqs():
    pat = r"100k_d(\d+)_flat/disk"
    points = [x for x in match if re.match(pat, x[0])]
    for i in range(len(points)):
        m = re.match(pat, points[i][0]).groups()
        # dim, seq_par
        points[i] = [int(m[0])] + points[i][1:]

    x = [10, 12, 14, 16, 18, 20, 22, 24, 26, 28, 30]

    y = [x[1] for x in points]
    y_el = [x[2] for x in points]
    y_eh = [x[3] for x in points]
    y_err = [y_el, y_eh]

    fig, ax = plt.subplots()
    plt.xlabel("Dimensions")
    plt.ylabel("Time taken (ms)")
    plt.xticks(x)

    ax.errorbar(x, lin, yerr=lin_err, label='Linear', capsize=5)
    ax.errorbar(x, seq, yerr=seq_err, label='Sequential', capsize=5)

    ax.legend()
    ax.grid()
    fig.savefig("seqs.png", dpi=300)

plot_seqs()