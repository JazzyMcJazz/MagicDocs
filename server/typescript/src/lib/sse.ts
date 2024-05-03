import sse from 'sse.js';

class SSE {

    private url: string;
    private options: object;
    private callback: () => void;

    constructor(url: string, options?: object, callback?: () => void) {
        this.url = url;
        this.options = options ?? {};
        this.callback = callback ?? (() => {});
    }

    public async send(data: object): Promise<void> {

    }
}