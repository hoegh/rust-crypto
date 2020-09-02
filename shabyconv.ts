/* A utility for convering the SHA Byte test vectors to something than is easier to
 * write Rust test cases from.
 */

// deno run --allow-read --allow-write shabyconv.ts

import { readFileStr } from 'https://deno.land/std/fs/read_file_str.ts';

function shabytest(testdata: string, render: (name: string, msg: string, digest: string)=>string) {
  var result: string = "";

  var name: string = "";
  var msg: string="";
  var digest: string="";

  let lines = testdata.split("\n");
  for (var line of lines) {
    line = line.trim();

    if (line.startsWith("Len =")) {
      name = "len"+line.slice(6);
    } else if (line.startsWith("Msg =")) {
      msg = line.slice(6);
      if (name == "Len = 0") {
        msg = "";
      }
    } else if (line.startsWith("MD =")) {
      digest = line.slice(5);

      result += render(name, msg, digest) + '\n';
    }

  }

  return result;
};

function renderRustTestCase(name: string, msg: string, digest: string) {
  return 'case::'+name+'( "'+msg+'", "'+digest+'" ),';
}

function convertTestFile(infile: string, outfile: string) {
  Deno.writeTextFileSync(outfile,
    shabytest(
      Deno.readTextFileSync(infile),
      renderRustTestCase));
}

convertTestFile("shabytetestvectors/SHA256ShortMsg.rsp", "shortcases.txt");
convertTestFile("shabytetestvectors/SHA256LongMsg.rsp", "longcases.txt");
