module.exports = {
  apps: [
    {
      name: 'UserData-API',
      script: './dist/server.js',
      watch: true,
      instances: '1',
      env: {
        NODE_ENV: 'development',
      },
      env_production: {
        NODE_ENV: 'production',
      },
    },
  ],

  deploy: {
    production: {
      user: 'SSH_USERNAME',
      host: 'SSH_HOSTMACHINE',
      ref: 'origin/main',
      repo: 'https://github.com/OfficialSirH/RESTful-C2SUserData.git',
      path: './',
      'pre-deploy-local': '',
      'post-deploy': 'npm install && tsc && pm2 reload dist/ecosystem.config.js --env production',
      'pre-setup': '',
    },
  },
};
