# animated-text
Refernce Implementation and Editor of experimental LRC file alternative


# Animated Text Storage Format. 



An experimental format is a compatible fromat that unifies features from various caption lyrics formats. At its core, it prioritizes readability and basic functionality, even at a minimal level of implementation knowledge. It's basically a virtual plain text file sysem with many different files with each of their own independent or co-dependent features. 



Reserved Characters:

[] is a reserved character to indicate data used by the format reader.
/  are generally used as seperators inside class properties.


Compared to other formats:

 
| Features     | Animated Text | AMLL | TTML | LRC | WebVTT | Srt |
|--------------|---------------|------|------|-----|--------|-----|
| Editor       | WIP           | [x]  | [x]  | [x] |        |     |
| Plain Text   | [x]           |      |      |     |        |     |
| Lines        | WIP           | [x]  | [x]  | [x] |        |     |
| Syllables    | WIP           | [x]  | [x]  |     |        |     |
| Duets        | WIP           | [-]  | [-]  |     |        |     |
| Idols        | WIP           |      |      |     |        |     |
| File Size    | LOW           | HIGH  | HIGH  | MINIMUM |  LOW      |     |
| Streamable   | WIP           | TBD  | TBD  |     |        |     |
| Positioning  | WIP           | TBD  | TBD  |     |        | [x] |
| Sel. Language | WIP           |    |    |     |        |  |
| **Interop.**     | **Animated Text** | **AMLL** | **TTML** | **LRC** | **WebVTT** | **Srt** |
|               |      |      |      |        |     |              |
| Animated Text  |   Full         |   WIP   |  WIP    |   Full (WIP)  |   WIP      | WIP
| AMLL				   |               |    Full  | Full     |      |        |
| TTML 				   |               |      |  Full    |  Full    |        |
| LRC				   |   Partial     |      |      |      |        |
| WebVTT			   |               |      |      |      |        |     |
| Srt				   |               |      |      |      |        |     |



<!--
https://github.com/amll-dev/applemusic-like-lyrics.git
https://amll-ttml-tool.stevexmh.net/-->
