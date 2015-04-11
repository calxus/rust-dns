results = ["my-server-1.txt", "my-server-2.txt", "isp-server-1.txt", "isp-server-2.txt"]

for result in results: 
	input_file = open(result, "r")
	names = list()
	found = False
	count = 0
	mal_packets = 0
	timeout = 0
	for line in input_file:
		if "Name:" in line:
			line = line.replace("Name:","")
			line = line. replace(" ", "")
			if line not in names:
				names.append(line)
				found = True
				continue
		if "Address:" in line:
			if "#53" not in line:
				if found == True:
					count += 1
					found = False
					continue
		if ";; connection timed out; no servers could be reached" in line:
			timeout += 1
			continue
		if ";; Warning: Message parser reports malformed message packet." in line:
			mal_packets += 1
			continue

		found = False
	print "Number of resolved domain names: ", count
	print "Timeouts: ", timeout
	print "Malformed Packets: ", mal_packets

