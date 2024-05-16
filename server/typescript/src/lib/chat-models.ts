import markdownit from 'markdown-it';
import shiki from '@shikijs/markdown-it';

export const renderMessage = (text: string, user?: string): number => {
    const chat = document.getElementById('chat-messages');
    const id = document.querySelectorAll('.message').length;

    let message = user
        ? userMessage(id, text, user)
        : botMessage(id, text);

    chat?.insertAdjacentHTML('afterbegin', message);

    return id;
};

export const updateMessage = (id: number, text: string) => {
    const message = document.getElementById(`message-${id}`);

    if (message) {
        message.innerHTML = text;
    }
}

const userMessage = (id: number, text: string, name: string) => {
    return `
        <div class="message">
            <div class="message-header">
                <svg fill="#fff" width="20px" height="20px" viewBox="0 0 56 56">
                    <path d="M 27.9999 51.9063 C 41.0546 51.9063 51.9063 41.0781 51.9063 28 C 51.9063 14.9453 41.0312 4.0937 27.9765 4.0937 C 14.8983 4.0937 4.0937 14.9453 4.0937 28 C 4.0937 41.0781 14.9218 51.9063 27.9999 51.9063 Z M 27.9999 35.9922 C 20.9452 35.9922 15.5077 38.5 13.1405 41.3125 C 9.9999 37.7968 8.1014 33.1328 8.1014 28 C 8.1014 16.9609 16.9140 8.0781 27.9765 8.0781 C 39.0155 8.0781 47.8983 16.9609 47.9219 28 C 47.9219 33.1563 46.0234 37.8203 42.8593 41.3359 C 40.4921 38.5234 35.0546 35.9922 27.9999 35.9922 Z M 27.9999 32.0078 C 32.4999 32.0547 36.0390 28.2109 36.0390 23.1719 C 36.0390 18.4375 32.4765 14.5 27.9999 14.5 C 23.4999 14.5 19.9140 18.4375 19.9609 23.1719 C 19.9843 28.2109 23.4765 31.9609 27.9999 32.0078 Z"/>
                </svg>
                <span class="message-user">${name}</span>
            </div>
            <div id="message-${id}" class="message-body">${text}</div>
        </div>
    `;
};

const botMessage = (id: number, message: string) => {
    let color = '#fff';

    return `
        <div class="message">
            <div class="message-header">
                <svg width="20px" height="20px" viewBox="0 0 400 400" fill="none">
                    <path
                        d="M97.8357 54.6682C177.199 59.5311 213.038 52.9891 238.043 52.9891C261.298 52.9891 272.24 129.465 262.683 152.048C253.672 173.341 100.331 174.196 93.1919 165.763C84.9363 156.008 89.7095 115.275 89.7095 101.301"
                        stroke="${color}" stroke-opacity="0.9" stroke-width="16" stroke-linecap="round" stroke-linejoin="round"
                    />
                    <path d="M98.3318 190.694C-10.6597 291.485 121.25 273.498 148.233 295.083" stroke="${color}" stroke-opacity="0.9" stroke-width="16" stroke-linecap="round" stroke-linejoin="round"/>
                    <path d="M98.3301 190.694C99.7917 213.702 101.164 265.697 100.263 272.898" stroke="${color}" stroke-opacity="0.9" stroke-width="16" stroke-linecap="round" stroke-linejoin="round"/>
                    <path d="M208.308 136.239C208.308 131.959 208.308 127.678 208.308 123.396" stroke="${color}" stroke-opacity="0.9" stroke-width="16" stroke-linecap="round" stroke-linejoin="round"/>
                    <path d="M177.299 137.271C177.035 133.883 177.3 126.121 177.3 123.396" stroke="${color}" stroke-opacity="0.9" stroke-width="16" stroke-linecap="round" stroke-linejoin="round"/>
                    <path d="M203.398 241.72C352.097 239.921 374.881 226.73 312.524 341.851" stroke="${color}" stroke-opacity="0.9" stroke-width="16" stroke-linecap="round" stroke-linejoin="round"/>
                    <path d="M285.55 345.448C196.81 341.85 136.851 374.229 178.223 264.504" stroke="${color}" stroke-opacity="0.9" stroke-width="16" stroke-linecap="round" stroke-linejoin="round"/>
                    <path d="M180.018 345.448C160.77 331.385 139.302 320.213 120.658 304.675" stroke="${color}" stroke-opacity="0.9" stroke-width="16" stroke-linecap="round" stroke-linejoin="round"/>
                    <path d="M218.395 190.156C219.024 205.562 219.594 220.898 219.594 236.324" stroke="${color}" stroke-opacity="0.9" stroke-width="16" stroke-linecap="round" stroke-linejoin="round"/>
                    <path d="M218.395 190.156C225.896 202.037 232.97 209.77 241.777 230.327" stroke="${color}" stroke-opacity="0.9" stroke-width="16" stroke-linecap="round" stroke-linejoin="round"/>
                    <path d="M80.1174 119.041C75.5996 120.222 71.0489 119.99 66.4414 120.41" stroke="${color}" stroke-opacity="0.9" stroke-width="16" stroke-linecap="round" stroke-linejoin="round"/>
                    <path d="M59.5935 109.469C59.6539 117.756 59.5918 125.915 58.9102 134.086" stroke="${color}" stroke-opacity="0.9" stroke-width="16" stroke-linecap="round" stroke-linejoin="round"/>
                    <path d="M277.741 115.622C281.155 115.268 284.589 114.823 287.997 114.255" stroke="${color}" stroke-opacity="0.9" stroke-width="16" stroke-linecap="round" stroke-linejoin="round"/>
                    <path d="M291.412 104.682C292.382 110.109 292.095 115.612 292.095 121.093" stroke="${color}" stroke-opacity="0.9" stroke-width="16" stroke-linecap="round" stroke-linejoin="round"/>
                    <path d="M225.768 116.466C203.362 113.993 181.657 115.175 160.124 118.568" stroke="${color}" stroke-opacity="0.9" stroke-width="16" stroke-linecap="round" stroke-linejoin="round"/>
                    </svg>
                <span class="message-user">Magic Docs</span>
            </div>
            <div id="message-${id}" class="message-body">${message}</div>
        </div>
    `;
}