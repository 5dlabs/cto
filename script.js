// Global state
let demoRunning = false;
let demoStep = 0;

// Demo terminal outputs
const demoSteps = [
    { delay: 1000, text: "🚀 Starting CTO agent deployment...", type: "info" },
    { delay: 800, text: "📋 Loading task: \"Build REST API for user management\"", type: "info" },
    { delay: 1200, text: "🤖 Rex agent initialized", type: "success" },
    { delay: 1000, text: "💻 Analyzing requirements and dependencies...", type: "typing" },
    { delay: 1500, text: "📦 Installing dependencies: express, cors, helmet, joi", type: "info" },
    { delay: 1000, text: "🏗️  Creating project structure...", type: "info" },
    { delay: 800, text: "   ├── src/routes/users.js", type: "file" },
    { delay: 300, text: "   ├── src/middleware/auth.js", type: "file" },
    { delay: 300, text: "   ├── src/models/User.js", type: "file" },
    { delay: 300, text: "   └── src/app.js", type: "file" },
    { delay: 1200, text: "⚡ Writing optimized API endpoints...", type: "typing" },
    { delay: 2000, text: "✅ Generated 5 RESTful endpoints with validation", type: "success" },
    { delay: 1000, text: "🧪 Tess agent taking over for testing...", type: "agent" },
    { delay: 1500, text: "🧪 Writing comprehensive test suite...", type: "typing" },
    { delay: 2000, text: "✅ Generated 47 unit tests (100% coverage)", type: "success" },
    { delay: 800, text: "✅ Generated 12 integration tests", type: "success" },
    { delay: 1000, text: "👁️  Cleo agent reviewing code quality...", type: "agent" },
    { delay: 1500, text: "🔍 Running ESLint, Prettier, and security scans...", type: "info" },
    { delay: 1200, text: "✅ Code quality: A+ (0 issues found)", type: "success" },
    { delay: 800, text: "📚 Morgan agent generating documentation...", type: "agent" },
    { delay: 1500, text: "📖 Creating API documentation and README...", type: "info" },
    { delay: 1200, text: "✅ Generated complete API docs with examples", type: "success" },
    { delay: 1000, text: "🔐 Cipher agent running security audit...", type: "agent" },
    { delay: 1200, text: "🛡️  Scanning for vulnerabilities and best practices...", type: "info" },
    { delay: 1000, text: "✅ Security score: 98/100 (enterprise ready)", type: "success" },
    { delay: 800, text: "🚀 Creating GitHub pull request...", type: "info" },
    { delay: 1200, text: "✅ PR #42 created: \"feat: Add user management API\"", type: "success" },
    { delay: 500, text: "📊 Summary:", type: "header" },
    { delay: 300, text: "   • 247 lines of production code", type: "stat" },
    { delay: 300, text: "   • 100% test coverage", type: "stat" },
    { delay: 300, text: "   • A+ code quality rating", type: "stat" },
    { delay: 300, text: "   • Enterprise security standards", type: "stat" },
    { delay: 300, text: "   • Complete documentation", type: "stat" },
    { delay: 500, text: "⏱️  Total time: 3 hours 42 minutes", type: "time" },
    { delay: 300, text: "💰 Cost savings: $47,000 vs traditional development", type: "savings" },
    { delay: 1000, text: "🎉 Mission accomplished! Ready for code review.", type: "complete" }
];

// Agent rotation
const agents = ['rex', 'cleo', 'tess'];
let currentAgentIndex = 0;

// DOM Elements
const demoOutput = document.getElementById('demo-output');
const waitlistModal = document.getElementById('waitlist-modal');

// Initialize on page load
document.addEventListener('DOMContentLoaded', function() {
    // Start agent rotation
    startAgentRotation();
    
    // Add smooth scrolling
    document.querySelectorAll('a[href^="#"]').forEach(anchor => {
        anchor.addEventListener('click', function (e) {
            e.preventDefault();
            const target = document.querySelector(this.getAttribute('href'));
            if (target) {
                target.scrollIntoView({ behavior: 'smooth' });
            }
        });
    });

    // Navbar scroll effect
    window.addEventListener('scroll', function() {
        const nav = document.querySelector('.nav');
        if (window.scrollY > 50) {
            nav.style.background = 'rgba(15, 15, 35, 0.98)';
        } else {
            nav.style.background = 'rgba(15, 15, 35, 0.95)';
        }
    });

    // Add typing animation to hero
    setTimeout(() => {
        const typingElements = document.querySelectorAll('.typing-animation');
        typingElements.forEach(el => {
            el.classList.add('typing-active');
        });
    }, 2000);
});

// Agent rotation functionality
function startAgentRotation() {
    setInterval(() => {
        // Deactivate current agent
        document.querySelectorAll('.agent-card').forEach(card => {
            card.classList.remove('active');
        });
        
        // Activate next agent
        currentAgentIndex = (currentAgentIndex + 1) % agents.length;
        const nextAgent = agents[currentAgentIndex];
        const nextCard = document.querySelector(`[data-agent="${nextAgent}"]`);
        if (nextCard) {
            nextCard.classList.add('active');
        }
    }, 4000);
}

// Demo functionality
function startDemo() {
    if (demoRunning) return;
    
    demoRunning = true;
    demoStep = 0;
    demoOutput.innerHTML = '';
    
    // Disable start button
    const startBtn = document.querySelector('.demo-button.primary');
    startBtn.textContent = '⏳ Running...';
    startBtn.disabled = true;
    
    runDemoStep();
}

function runDemoStep() {
    if (demoStep >= demoSteps.length) {
        completDemo();
        return;
    }
    
    const step = demoSteps[demoStep];
    
    setTimeout(() => {
        addDemoLine(step.text, step.type);
        demoStep++;
        
        if (demoRunning) {
            runDemoStep();
        }
    }, step.delay);
}

function addDemoLine(text, type) {
    const line = document.createElement('div');
    line.className = `output-line ${type}`;
    
    // Add type-specific styling
    switch (type) {
        case 'success':
            line.style.color = '#4ecdc4';
            break;
        case 'info':
            line.style.color = '#45b7d1';
            break;
        case 'agent':
            line.style.color = '#ff6b6b';
            line.style.fontWeight = 'bold';
            break;
        case 'typing':
            line.classList.add('typing-animation');
            break;
        case 'file':
            line.style.color = '#96ceb4';
            line.style.marginLeft = '20px';
            break;
        case 'stat':
            line.style.color = '#feca57';
            line.style.marginLeft = '20px';
            break;
        case 'time':
            line.style.color = '#ff6b6b';
            line.style.fontWeight = 'bold';
            break;
        case 'savings':
            line.style.color = '#4ecdc4';
            line.style.fontWeight = 'bold';
            break;
        case 'complete':
            line.style.color = '#4ecdc4';
            line.style.fontWeight = 'bold';
            line.style.fontSize = '1.1em';
            break;
        case 'header':
            line.style.color = '#ffffff';
            line.style.fontWeight = 'bold';
            break;
    }
    
    line.textContent = text;
    demoOutput.appendChild(line);
    
    // Auto scroll to bottom
    demoOutput.scrollTop = demoOutput.scrollHeight;
}

function resetDemo() {
    demoRunning = false;
    demoStep = 0;
    demoOutput.innerHTML = '<div class="output-line">Demo reset. Click "Start Demo" to begin.</div>';
    
    // Re-enable start button
    const startBtn = document.querySelector('.demo-button.primary');
    startBtn.textContent = '▶️ Start Demo';
    startBtn.disabled = false;
}

function skipDemo() {
    if (!demoRunning) return;
    
    // Show final state immediately
    demoOutput.innerHTML = '';
    const finalSteps = demoSteps.slice(-10); // Last 10 steps
    
    finalSteps.forEach((step, index) => {
        setTimeout(() => {
            addDemoLine(step.text, step.type);
            if (index === finalSteps.length - 1) {
                completDemo();
            }
        }, index * 100);
    });
}

function completDemo() {
    demoRunning = false;
    
    // Re-enable start button
    const startBtn = document.querySelector('.demo-button.primary');
    startBtn.textContent = '🔄 Run Again';
    startBtn.disabled = false;
    
    // Add celebration effect
    setTimeout(() => {
        addDemoLine('🎊 Want to see this with your own code? Try CTO now!', 'complete');
    }, 1000);
}

// Utility functions
function scrollToDemo() {
    document.getElementById('demo').scrollIntoView({ behavior: 'smooth' });
}

function copyCommand() {
    const commandText = 'curl -sSL https://demo.5dlabs.com | bash';
    navigator.clipboard.writeText(commandText).then(() => {
        // Show feedback
        const copyBtn = document.querySelector('.copy-button');
        const originalText = copyBtn.textContent;
        copyBtn.textContent = '✅';
        copyBtn.style.color = '#4ecdc4';
        
        setTimeout(() => {
            copyBtn.textContent = originalText;
            copyBtn.style.color = '';
        }, 2000);
    }).catch(err => {
        console.error('Failed to copy: ', err);
    });
}

// Modal functions
function openWaitlist() {
    waitlistModal.style.display = 'block';
    document.body.style.overflow = 'hidden';
}

function closeWaitlist() {
    waitlistModal.style.display = 'none';
    document.body.style.overflow = 'auto';
}

// Close modal when clicking outside
window.addEventListener('click', function(event) {
    if (event.target === waitlistModal) {
        closeWaitlist();
    }
});

// Form submissions
function subscribeNewsletter(event) {
    event.preventDefault();
    const email = event.target.querySelector('input[type="email"]').value;
    
    // Simulate API call
    const button = event.target.querySelector('.newsletter-button');
    const originalText = button.textContent;
    button.textContent = 'Subscribing...';
    button.disabled = true;
    
    setTimeout(() => {
        button.textContent = '✅ Subscribed!';
        button.style.background = '#4ecdc4';
        
        // Show success message
        alert('Thanks for subscribing! You\'ll be the first to know about new features.');
        
        // Reset form
        event.target.reset();
        
        setTimeout(() => {
            button.textContent = originalText;
            button.style.background = '';
            button.disabled = false;
        }, 3000);
    }, 1500);
}

function submitWaitlist(event) {
    event.preventDefault();
    const form = event.target;
    const formData = new FormData(form);
    
    // Simulate API call
    const button = form.querySelector('.primary-button');
    const originalText = button.textContent;
    button.textContent = 'Joining...';
    button.disabled = true;
    
    setTimeout(() => {
        button.textContent = '🎉 You\'re In!';
        button.style.background = '#4ecdc4';
        
        // Show success message
        setTimeout(() => {
            closeWaitlist();
            alert('Welcome to the waitlist! We\'ll be in touch soon with early access.');
            form.reset();
            
            button.textContent = originalText;
            button.style.background = '';
            button.disabled = false;
        }, 2000);
    }, 1500);
}

// Easter egg: Konami code
let konamiCode = [];
const correctCode = [
    'ArrowUp', 'ArrowUp', 'ArrowDown', 'ArrowDown',
    'ArrowLeft', 'ArrowRight', 'ArrowLeft', 'ArrowRight',
    'KeyB', 'KeyA'
];

document.addEventListener('keydown', function(event) {
    konamiCode.push(event.code);
    konamiCode = konamiCode.slice(-correctCode.length);
    
    if (konamiCode.join('') === correctCode.join('')) {
        // Easter egg activated
        document.body.style.filter = 'hue-rotate(180deg)';
        setTimeout(() => {
            document.body.style.filter = '';
        }, 3000);
        
        // Add special message to demo
        if (demoOutput) {
            addDemoLine('🎮 Easter egg activated! You found the secret code!', 'complete');
        }
    }
});

// Performance monitoring
function trackEvent(eventName, properties = {}) {
    // In a real app, this would send to analytics
    console.log('Event:', eventName, properties);
}

// Track key interactions
document.addEventListener('click', function(event) {
    const element = event.target;
    
    if (element.classList.contains('primary-button')) {
        trackEvent('CTA_Click', { button: element.textContent });
    } else if (element.classList.contains('demo-button')) {
        trackEvent('Demo_Interaction', { action: element.textContent });
    } else if (element.classList.contains('nav-link')) {
        trackEvent('Navigation', { link: element.textContent });
    }
});

// GitHub stats integration (would be real API in production)
function updateGitHubStats() {
    // Simulated stats - in production, these would come from GitHub API
    const stats = {
        stars: '2.3k',
        forks: '456',
        contributors: '23'
    };
    
    // Update any GitHub stat displays
    document.querySelectorAll('[data-stat]').forEach(element => {
        const statType = element.getAttribute('data-stat');
        if (stats[statType]) {
            element.textContent = stats[statType];
        }
    });
}

// Initialize GitHub stats
setTimeout(updateGitHubStats, 2000);

// Smooth reveal animations on scroll
const observerOptions = {
    threshold: 0.1,
    rootMargin: '0px 0px -50px 0px'
};

const observer = new IntersectionObserver(function(entries) {
    entries.forEach(entry => {
        if (entry.isIntersecting) {
            entry.target.style.opacity = '1';
            entry.target.style.transform = 'translateY(0)';
        }
    });
}, observerOptions);

// Observe elements for animation
document.addEventListener('DOMContentLoaded', function() {
    const animatedElements = document.querySelectorAll('.feature-card, .pricing-card, .demo-stat');
    
    animatedElements.forEach(el => {
        el.style.opacity = '0';
        el.style.transform = 'translateY(30px)';
        el.style.transition = 'all 0.6s ease';
        observer.observe(el);
    });
});

// Preload critical resources
function preloadResources() {
    // Preload fonts and critical images
    const link = document.createElement('link');
    link.rel = 'preload';
    link.as = 'font';
    link.type = 'font/woff2';
    link.crossOrigin = 'anonymous';
    link.href = 'https://fonts.gstatic.com/s/inter/v12/UcCO3FwrK3iLTeHuS_fvQtMwCp50KnMw2boKoduKmMEVuLyfAZ9hiA.woff2';
    document.head.appendChild(link);
}

preloadResources();
