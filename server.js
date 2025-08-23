const http = require('http');
const fs = require('fs');
const path = require('path');
const url = require('url');

const port = process.env.PORT || 3000;

// MIME types
const mimeTypes = {
    '.html': 'text/html',
    '.js': 'application/javascript',
    '.css': 'text/css',
    '.json': 'application/json',
    '.png': 'image/png',
    '.jpg': 'image/jpg',
    '.gif': 'image/gif',
    '.svg': 'image/svg+xml',
    '.wav': 'audio/wav',
    '.mp4': 'video/mp4',
    '.woff': 'application/font-woff',
    '.ttf': 'application/font-ttf',
    '.eot': 'application/vnd.ms-fontobject',
    '.otf': 'application/font-otf',
    '.wasm': 'application/wasm'
};

const server = http.createServer((req, res) => {
    const parsedUrl = url.parse(req.url);
    let pathname = parsedUrl.pathname;
    
    // Default to index.html
    if (pathname === '/') {
        pathname = '/index.html';
    }
    
    // Special endpoints
    if (pathname === '/api/demo' && req.method === 'POST') {
        handleDemoAPI(req, res);
        return;
    }
    
    if (pathname === '/api/waitlist' && req.method === 'POST') {
        handleWaitlistAPI(req, res);
        return;
    }
    
    if (pathname === '/api/newsletter' && req.method === 'POST') {
        handleNewsletterAPI(req, res);
        return;
    }
    
    // Health check endpoint
    if (pathname === '/health') {
        res.writeHead(200, { 'Content-Type': 'application/json' });
        res.end(JSON.stringify({ status: 'ok', timestamp: new Date().toISOString() }));
        return;
    }
    
    // Serve static files
    const filePath = path.join(__dirname, pathname);
    
    fs.readFile(filePath, (err, data) => {
        if (err) {
            if (err.code === 'ENOENT') {
                // 404 - File not found
                res.writeHead(404, { 'Content-Type': 'text/html' });
                res.end(`
                    <html>
                        <head><title>404 - Page Not Found</title></head>
                        <body style="font-family: Arial; text-align: center; padding: 2rem; background: #0f0f23; color: white;">
                            <h1>🚀 5D Labs</h1>
                            <h2>404 - Page Not Found</h2>
                            <p>The page you're looking for doesn't exist.</p>
                            <a href="/" style="color: #4ecdc4;">← Back to Home</a>
                        </body>
                    </html>
                `);
            } else {
                // 500 - Internal server error
                res.writeHead(500, { 'Content-Type': 'text/plain' });
                res.end('Internal Server Error');
            }
            return;
        }
        
        // Get file extension
        const ext = path.extname(filePath).toLowerCase();
        const contentType = mimeTypes[ext] || 'application/octet-stream';
        
        // Set security headers
        res.setHeader('X-Content-Type-Options', 'nosniff');
        res.setHeader('X-Frame-Options', 'DENY');
        res.setHeader('X-XSS-Protection', '1; mode=block');
        
        // Set caching headers for static assets
        if (['.css', '.js', '.png', '.jpg', '.gif', '.svg'].includes(ext)) {
            res.setHeader('Cache-Control', 'public, max-age=31536000'); // 1 year
        }
        
        res.writeHead(200, { 'Content-Type': contentType });
        res.end(data);
    });
});

// API Handlers
function handleDemoAPI(req, res) {
    let body = '';
    req.on('data', chunk => {
        body += chunk.toString();
    });
    
    req.on('end', () => {
        try {
            const data = JSON.parse(body);
            console.log('Demo request:', data);
            
            // Simulate demo response
            const response = {
                success: true,
                message: 'Demo started successfully!',
                demoId: 'demo_' + Date.now(),
                estimatedTime: '3-5 minutes'
            };
            
            res.writeHead(200, { 
                'Content-Type': 'application/json',
                'Access-Control-Allow-Origin': '*',
                'Access-Control-Allow-Methods': 'POST, GET, OPTIONS',
                'Access-Control-Allow-Headers': 'Content-Type'
            });
            res.end(JSON.stringify(response));
        } catch (error) {
            res.writeHead(400, { 'Content-Type': 'application/json' });
            res.end(JSON.stringify({ error: 'Invalid JSON' }));
        }
    });
}

function handleWaitlistAPI(req, res) {
    let body = '';
    req.on('data', chunk => {
        body += chunk.toString();
    });
    
    req.on('end', () => {
        try {
            const data = JSON.parse(body);
            console.log('Waitlist signup:', data.email, data.company);
            
            // In a real app, this would save to database
            const response = {
                success: true,
                message: 'Successfully added to waitlist!',
                position: Math.floor(Math.random() * 500) + 100 // Fake position
            };
            
            res.writeHead(200, { 
                'Content-Type': 'application/json',
                'Access-Control-Allow-Origin': '*',
                'Access-Control-Allow-Methods': 'POST, GET, OPTIONS',
                'Access-Control-Allow-Headers': 'Content-Type'
            });
            res.end(JSON.stringify(response));
        } catch (error) {
            res.writeHead(400, { 'Content-Type': 'application/json' });
            res.end(JSON.stringify({ error: 'Invalid JSON' }));
        }
    });
}

function handleNewsletterAPI(req, res) {
    let body = '';
    req.on('data', chunk => {
        body += chunk.toString();
    });
    
    req.on('end', () => {
        try {
            const data = JSON.parse(body);
            console.log('Newsletter signup:', data.email);
            
            const response = {
                success: true,
                message: 'Successfully subscribed to newsletter!'
            };
            
            res.writeHead(200, { 
                'Content-Type': 'application/json',
                'Access-Control-Allow-Origin': '*',
                'Access-Control-Allow-Methods': 'POST, GET, OPTIONS',
                'Access-Control-Allow-Headers': 'Content-Type'
            });
            res.end(JSON.stringify(response));
        } catch (error) {
            res.writeHead(400, { 'Content-Type': 'application/json' });
            res.end(JSON.stringify({ error: 'Invalid JSON' }));
        }
    });
}

// Handle CORS preflight requests
server.on('request', (req, res) => {
    if (req.method === 'OPTIONS') {
        res.writeHead(200, {
            'Access-Control-Allow-Origin': '*',
            'Access-Control-Allow-Methods': 'POST, GET, OPTIONS',
            'Access-Control-Allow-Headers': 'Content-Type'
        });
        res.end();
    }
});

// Start server
server.listen(port, () => {
    console.log(`
🚀 5D Labs Landing Page Server

Server running at: http://localhost:${port}
    
Ready to showcase autonomous AI development!
    
To expose this with ngrok:
  ngrok http ${port} --domain=demo.5dlabs.com
    
Or with a random subdomain:
  ngrok http ${port}
    `);
});

// Graceful shutdown
process.on('SIGTERM', () => {
    console.log('Received SIGTERM, shutting down gracefully...');
    server.close(() => {
        console.log('Server closed');
        process.exit(0);
    });
});

process.on('SIGINT', () => {
    console.log('\nReceived SIGINT, shutting down gracefully...');
    server.close(() => {
        console.log('Server closed');
        process.exit(0);
    });
});
