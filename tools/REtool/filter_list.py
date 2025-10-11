def run(input_file, output_file, patterns):
    with open(input_file, 'r') as infile, open(output_file, 'w') as outfile:
        for line in infile:
            path = line.rstrip('\n')

            if any(path.endswith(pattern) for pattern in patterns):
                outfile.write(line)

if __name__ == "__main__":
    run(
        "MHWs.source.list",
        "MHWs.list",
        [".user.3"],
    )