# Hugo Website Design for Babylon

Based on your game's focus on dialectical materialism and the brutalist aesthetic shown in your GUI code, I'll suggest a structure that reflects these themes while being practical for marketing the game.

## Recommended Directory Structure

```plaintext
babylon-dotcom/
├── archetypes/        # Templates for new content
├── assets/           # Raw assets that need processing (SCSS, JS)
├── config.toml       # Main Hugo configuration
├── content/         # Main content files
│   ├── _index.md    # Homepage
│   ├── about/       # About the game
│   ├── features/    # Game features/mechanics
│   ├── blog/        # Dev blog posts
│   ├── docs/        # Game documentation
│   └── media/       # Screenshots, videos
├── layouts/         # HTML templates
│   ├── _default/
│   ├── partials/
│   └── shortcodes/
├── static/          # Static assets (images, css, js)
└── themes/          # Theme files
```

## Reasoning for this Structure

### 1. Content Organization
- Separating content into distinct sections (about, features, blog, docs) makes it easy to manage and organize game information
- Blog section allows for dev updates and maintaining community engagement
- Documentation section can introduce game concepts like contradictions and dialectical materialism

### 2. Asset Management
- `assets/` for SCSS/JS that needs processing
- `static/` for ready-to-use assets
- This separation helps maintain a clean build process

### 3. Theme Development
- Custom theme can reflect your game's brutalist/industrial aesthetic
- Partials and shortcodes allow for reusable components

## Implementation Steps

### Initial Setup

```bash
# Create the site directory and initialize Hugo
cd ~/projects
hugo new site babylon-dotcom
cd babylon-dotcom

# Initialize git repository
git init

# Create basic content structure
mkdir -p content/{about,features,blog,docs,media}

# Add the Ananke theme as a git submodule
git submodule add https://github.com/theNewDynamic/gohugo-theme-ananke.git themes/ananke
```

### Basic Configuration

Create `config.toml`:

```toml
baseURL = 'http://example.org/'
languageCode = 'en-us'
title = 'Babylon'
theme = 'ananke'

[params]
  description = "A dialectical materialist simulation game"
  featured_image = '/images/hero.jpg'
```

### Initial Content

Create `content/_index.md`:

```markdown
---
title: "Babylon"
description: "Explore the forces of historical materialism"
---

Welcome to Babylon, a simulation game where dialectical contradictions shape the course of history. Navigate complex social forces, manage economic systems, and witness the transformation of society through the lens of historical materialism.
```

### Preview Your Site

```bash
hugo server -D
```

Visit http://localhost:1313 to view your site.

## Next Steps

## Hosting Options

### 1. GitHub Pages (Free)
- Host directly from your GitHub repository
- Easy to set up, just push your `public/` directory to a `gh-pages` branch
- Supports custom domains
- Reliable and fast, backed by GitHub's infrastructure

### 2. Netlify (Free Tier)
- Offers continuous deployment from your Git repository
- Automatically builds your site whenever you push changes
- Provides a global CDN for fast content delivery
- Includes free SSL/TLS certificates
- Offers serverless functions and form handling

### 3. Vercel (Free Tier)
- Similar to Netlify, with continuous deployment and global CDN
- Optimized for performance, with automatic code splitting and caching
- Supports serverless functions and API routes
- Easy to set up and deploy from your Git repository

### 4. Render (Free Tier)
- Offers static site hosting with continuous deployment
- Supports custom domains and free SSL/TLS
- Provides a global CDN for fast content delivery
- Includes a generous free tier with unlimited sites and bandwidth

### 5. Surge.sh (Free)
- Simple, single-command deployment for static sites
- Supports custom domains and free SSL/TLS
- Offers basic authentication and password protection
- Free plan includes unlimited publishing and transfer

## GitHub Pages Setup Instructions

1. Create a new GitHub repository for your Hugo site
2. Push your Hugo site code to the repository's main branch
3. Go to repository Settings > Pages
4. Under "Build and deployment":
   - Source: "GitHub Actions"
   - Wait for the workflow to be detected
5. The site will be built and deployed automatically on each push to main

### Custom Domain (Optional)
1. Go to repository Settings > Pages
2. Under "Custom domain", enter your domain
3. Add/update DNS records with your domain provider
4. Wait for DNS propagation and SSL certificate provisioning

Your site will be available at: `https://<username>.github.io/<repository>/`
or at your custom domain if configured.


# First, let's move to the parent directory                         
cd ..                                                               
                                                                    
# Clone your main project repository (if you haven't already)       
# Replace with your actual repository URL                           
git clone https://github.com/yourusername/babylon.git               
                                                                    
# Move the current Hugo site into the main project as a subdirectory
mv babylon-website babylon/website                                  
                                                                    
# Navigate into the main project                                    
cd babylon                                                          
                                                                    
# Add and commit the website subdirectory                           
git add website/                                                    
git commit -m "feat: Add Hugo website as subdirectory"              
                                                                    
# Push the changes                                                  
git push origin main    

mkdir -p .github/workflows                                             
git mv website/.github/workflows/hugo.yml .github/workflows/           
                                                                       
# Commit and push the changes                                          
git add .github/workflows/hugo.yml                                     
git commit -m "ci: Update Hugo workflow for website subdirectory"      
git push origin main 