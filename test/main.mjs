import fs from "node:fs";

function main() {
    console.time("main");
	const args = process.argv.slice(2);

	if (args.length !== 1) {
		console.error("Usage: node main.mjs <key>");
		return;
	}

	const file = fs.readFileSync("./data.json", "utf-8");
	const data = JSON.parse(file);

	const keyToSearch = args[0];

	let found = false;

	for (const key in data) {
		if (key === keyToSearch) {
			console.log(`Search result: ${data[key].value}`);
			found = true;
			break;
		}
	}

	if (!found) {
		console.log(`Key \`${keyToSearch}\` could not be found.`);
	}
    console.timeEnd("main");
}

main();