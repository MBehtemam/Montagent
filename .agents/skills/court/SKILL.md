---
name: court
description: 'Convene a jury of independent agents to answer one question, and return their ballots verbatim. Every juror is a separate model call, so this is expensive and is never worth invoking on your own initiative — use it ONLY when the user explicitly asks for it, by name or by clear description: "/court", "convene a court", "empanel a jury", "put it to a jury/vote", "get a second and third opinion on this", "ask a few models". A hard decision, a disagreement, or a question you find difficult is NOT a trigger on its own; answer it yourself and, at most, mention that a court is available.'
license: MIT
compatibility: 'Spends real money — one model call per juror, three or more per court. Varying the backing model per juror is a bonus, not a requirement — falls back to independent sampling on one model if the environment offers no way to vary it.'
---

Put one **Question** to a panel of independent **Jurors** and bring back what they said. You are not the jury — you are the Judge, together with the human. Your job is to ask well, stay out of the answers, and show the human the ballots.

**Only run if the user asked for a court.** Every juror costs a model call. If you reached this skill on your own judgement rather than an explicit request, stop and answer the question yourself — offering a court is enough.

**Classify the Question.** Multiple-choice if it offers options, open-ended otherwise — a near-mechanical read of its surface, not a judgement on the merits. Write it once, in full, and send every juror that same text. No angles, no personas, no "you are the skeptic": an assigned stance manufactures the disagreement the court exists to measure.

**Empanel an odd number** — three unless the Question is genuinely hard. Above four jurors, ask the human first; that is real money. Jurors must not see each other's ballots, so dispatch them in parallel and never relay one juror's answer to another.

**Vary what backs them where you can.** Use whatever mechanism your own environment actually provides for giving different subagents different models or backends — a per-agent model setting, sibling CLIs you know to be installed, or anything else at hand. Don't invent a CLI invocation, flag, or model name you haven't confirmed exists in this environment; a guessed model string fails outright and costs the juror. If you can't vary the backing model, still empanel independent jurors — independent sampling has real value — but the ballots will show it: same-model agreement is weak evidence of anything.

Instruct each juror to answer and nothing else — no editing, no tools, no recommendations to you — in exactly this block, naming the model that backs them so the human can see how varied the panel really was:

```
🗳️ **Juror <n>** (<the model backing them>) — **VOTE: <the chosen option, or a one-line answer>**

**Reasoning:** <why>
**Trade-offs:** <what it costs — or, for a multiple-choice Question, why not the others>
```

A juror may reject the framing outright: if the options are all bad or the Question is wrong, saying so is a valid ballot, not a failed one.

**Report the ballots verbatim.** Every one, in full, to the human — your summary is not a substitute for them. Table them only when the answers are commensurable, which in practice means the multiple-choice case; open-ended answers usually sit on different axes, and the first thing a cell truncates is the reasoning. Do not tally, do not call a split a hung jury, and never convene a second round to break a tie: a split is a valid verdict.

Then give your own read as the Judge — clearly marked as yours, after the ballots, never in place of them. **Stop there.** The Court shows the human what the jurors said and what you make of it; it does not decide, and it does not act on the verdict. The human decides. You are done when the ballots and your read are in front of them.
