class Rusher {
    constructor({ host }) {
        this.socket = this.connect(host)
        this.listeners = {}

        this.subscribe('subscribe', ({ event }) => {
            console.log(`Subscribed to ${event}`)
        })
    }

    connect(host) {
        const socket = new WebSocket(host)

        socket.addEventListener('open', () => {
            for (let event in this.listeners) {
                this.push({ event: 'subscribe', payload: { event } })
            }
        })

        socket.addEventListener('close', () => {
            console.log('Connection closed reconnecting...')

            setTimeout(() => {
                this.socket = this.connect(host)
            }, 5000)
        })

        socket.addEventListener('message', (event) => {
            try {
                let { name, data } = JSON.parse(event.data)

                if (!this.listeners[name]) {
                    return
                }

                this.listeners[name].forEach(listener => listener(data))
            } catch (error) {
                console.error(error)
            }
        })
        
        return socket
    }

    subscribe(event, callback) {
        if (!this.listeners[event]) {
            this.listeners[event] = []
        }

        this.listeners[event].push(callback)
    }

    unsubscribe(event, callback) {
        if (!this.listeners[event]) {
            return
        }

        this.listeners[event] = this.listeners[event].filter(listener => listener !== callback)
    }

    push({event, payload}) {
        this.socket.send(JSON.stringify({ event, payload }))
    }
}
