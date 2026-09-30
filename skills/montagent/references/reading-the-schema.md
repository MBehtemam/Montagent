# Reading the schema

<!-- workaround: #520 · replaced by: a schema served in pieces that fit a tool result -->

`montagent://schema.json` is over 70 KB, which is more than a tool result holds. Your client will cut it off or save it to a file. Either way, pull out only the part your question needs, and read that.

1. Fetch the resource once. If the client saves it to a file, keep the path. If it truncates, write the resource's text to a file yourself.
2. Query the file with Python for the one part you need:

   ```sh
   python3 -c 'import json,sys; s=json.load(open(sys.argv[1])); d=s["$defs"]; print(json.dumps(d[sys.argv[2]], indent=1))' <schema file> Highlight
   ```

   To list each element type with its keys and required keys:

   ```sh
   python3 -c 'import json,sys; s=json.load(open(sys.argv[1])); [print(o["properties"]["type"]["const"], sorted(o["properties"]), o["required"]) for o in s["$defs"]["Element"]["oneOf"]]' <schema file>
   ```

## Where to look

| Question | Where it lives |
|---|---|
| Top-level project keys | `properties` |
| An element type's keys, and which ones are required | `$defs.Element.oneOf`: one entry per `type` |
| Effect parameters | `$defs.Effect.oneOf`: one entry per `name` |
| Easing names and bezier limits | `$defs.Ease`, `$defs.EaseName` |
| Keyframe records | `$defs.Animatable`, `$defs.FirstKeyframe`, `$defs.Keyframe` (the numbered copies are the same shape for other value types) |
| Text runs and timed colour | `$defs.Run`, `$defs.Highlight` |

Every `description` in the schema states its own rule. Read the description on the key you are about to write.
