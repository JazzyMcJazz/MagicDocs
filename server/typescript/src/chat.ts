import { toggleExpandedOnClick, toggleOpenOnClick } from "./lib/chat-panel";
import { chatSse } from "./lib/chat-sse";

htmx.onLoad(() => {
    toggleExpandedOnClick();
    toggleOpenOnClick();
    chatSse();
});
