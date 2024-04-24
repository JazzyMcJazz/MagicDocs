FROM mcr.microsoft.com/playwright:v1.43.1-jammy

# Set the working directory in the container
WORKDIR /app

# Copy package.json and package-lock.json (if available)
COPY package.json package-lock.json* ./

# Install dependencies, including Playwright
RUN npm install --no-optional && npm cache clean --force

# Copy the rest of your application code
COPY . .

# Command to run your tests
CMD ["npm", "run", "test:ci"]