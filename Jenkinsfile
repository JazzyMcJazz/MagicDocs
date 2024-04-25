pipeline {
    agent any

    environment {
        SSH_TARGET = credentials('DEPLOY_SSH_TARGET')
    }

    stages {
        stage('Build Images') {
            steps {
                script {
                    echo 'Building Docker images'
                    sh 'docker build -t pgvector:latest database/'
                    sh 'docker build -t keycloak:latest keycloak/'
                    sh 'docker build -t magicdocs:latest server/'
                }
            }
        }
        stage('Run E2E Tests') {
            environment {
                PG_PASS = credentials('PG_PASS')
                KEYCLOAK_ADMIN_PASSWORD = credentials('KEYCLOAK_ADMIN_PASSWORD')
                KEYCLOAK_CLIENT_SECRET = credentials('KEYCLOAK_CLIENT_SECRET')
                TEST_USER_PASSWORD = credentials('TEST_USER_PASSWORD')
            }
            steps {
                // Build the Playwright test image
                script {
                    echo 'Building Docker image for Playwright tests'
                    sh 'docker build -t magicdocs_playwright:latest e2e/'
                }

                // Setup the test environment
                script {
                    echo 'Creating test environment'
                    try {
                        sh 'docker network create magicdocs_test_net'
                    } catch (Exception e) {
                        echo "Failed to create magicdocs_test_net - ${e.getMessage()}"
                    }

                    sh 'docker run -d --rm --name magicdocs_test_db \
                        --network magicdocs_test_net \
                        --network-alias db \
                        -e POSTGRES_PASSWORD=postgres \
                        -e PG_PASS=postgres \
                        -e KC_DB_USER=unused \
                        -e KC_DB_PASS=unused \
                        -e MD_DB_USER=magicdocs \
                        -e MD_DB_PASS=magicdocs \
                        -v magicdocs_test_db:/var/lib/postgresql/data \
                        --health-cmd=\'pg_isready -U postgres -d magicdocs\' \
                        --health-start-period=10s \
                        --health-start-interval=5s \
                        --health-interval=5m \
                        --health-timeout=10s \
                        --health-retries=3 \
                        pgvector:latest'

                    // Check the health status of the database container
                    def healthy = false
                    def retries = 0
                    while (!healthy && retries < 10) { // Timeout after 30 checks
                        sleep 1 // Wait 10 seconds before each check
                        def status = sh(script: "docker inspect --format='{{.State.Health.Status}}' magicdocs_test_db", returnStdout: true).trim()
                        if (status == "healthy") {
                            healthy = true
                            echo 'Database is ready.'
                        } else {
                            retries++
                            echo "Waiting for database to be healthy... Attempt ${retries}"
                        }
                    }

                    if (!healthy) {
                        error 'Database did not become healthy in time'
                    }

                    sh 'docker run -d --rm --name magicdocs_test_server \
                        --network magicdocs_test_net \
                        --network-alias server \
                        -e RUST_ENV=test \
                        -e DATABASE_URL=postgres://magicdocs:magicdocs@db:5432/magicdocs \
                        -e KEYCLOAK_INTERNAL_ADDR=https://kc.treeleaf.dev \
                        -e KEYCLOAK_EXTERNAL_ADDR=https://kc.treeleaf.dev \
                        -e KEYCLOAK_USER=admin \
                        -e KEYCLOAK_PASSWORD=$KEYCLOAK_ADMIN_PASSWORD \
                        -e KEYCLOAK_REALM=magicdocs \
                        -e KEYCLOAK_CLIENT=magicdocs \
                        -e KEYCLOAK_CLIENT_SECRET=$KEYCLOAK_CLIENT_SECRET \
                        -p 3000:3000 \
                        magicdocs:latest'
                }

                // Run the Playwright tests and save the results
                script {
                    sh 'docker run -d --name magicdocs_test_playwright \
                        --network magicdocs_test_net \
                        -e HOST_URL=http://server:3000 \
                        -e TEST_USER_USERNAME=tester \
                        -e TEST_USER_PASSWORD=$TEST_USER_PASSWORD \
                        magicdocs_playwright:latest'

                    def exitCode = sh(script: 'docker wait magicdocs_test_playwright', returnStatus: true, returnStdout: false)
                    echo "Playwright tests exited with code ${exitCode}"

                    sh 'docker cp magicdocs_test_playwright:/app/playwright-report/index.html .'

                    if (exitCode != 0) {
                        error 'Playwright tests failed'
                    }
                }
            }
            post {
                always {
                    echo 'Cleaning up test environment'
                    script {
                        try {
                            archiveArtifacts artifacts: 'index.html',
                                allowEmptyArchive: true
                        } catch (Exception e) {
                            echo "Failed to archive Playwright test results - ${e.getMessage()}"
                        }

                        try {
                            sh 'docker rm -f magicdocs_test_playwright'
                        } catch (Exception e) {
                            echo "Failed to remove magicdocs_test_playwright - ${e.getMessage()}"
                        }

                        try {
                            sh 'docker stop magicdocs_test_server'
                        } catch (Exception e) {
                            echo "Failed to stop magicdocs_test_server - ${e.getMessage()}"
                        }

                        try {
                            sh 'docker stop magicdocs_test_db'
                        } catch (Exception e) {
                            echo "Failed to stop magicdocs_test_db - ${e.getMessage()}"
                        }

                        try {
                            sh 'docker network rm magicdocs_test_net'
                        } catch (Exception e) {
                            echo "Failed to remove magicdocs_test_net - ${e.getMessage()}"
                        }

                        try {
                            sh 'docker rmi magicdocs_playwright:latest'
                        } catch (Exception e) {
                            echo "Failed to remove magicdocs_playwright:latest - ${e.getMessage()}"
                        }

                        try {
                            sh 'docker volume rm magicdocs_test_db'
                        } catch (Exception e) {
                            echo "Failed to remove magicdocs_test_db volume - ${e.getMessage()}"
                        }
                    }
                }
            }
        }
        stage('Deploy to Production') {
            environment {
                PG_PASS = credentials('PG_PASS')
                KC_DB_PASS = credentials('KC_DB_PASS')
                MD_DB_PASS = credentials('MD_DB_PASS')
                KEYCLOAK_ADMIN_PASSWORD = credentials('KEYCLOAK_ADMIN_PASSWORD')
                KEYCLOAK_CLIENT_SECRET = credentials('KEYCLOAK_CLIENT_SECRET')
            }
            steps {
                echo 'Saving Docker images'
                sh 'docker save -o pgvector.tar pgvector:latest'
                sh 'docker save -o keycloak.tar keycloak:latest'
                sh 'docker save -o magicdocs.tar magicdocs:latest'

                withCredentials(bindings: [sshUserPrivateKey(credentialsId: 'DeploymentTargetServer', keyFileVariable: 'SSH_KEY')]) {
                    echo 'Copying Docker images to production server'
                    sh 'scp -i $SSH_KEY pgvector.tar $SSH_TARGET:~'
                    sh 'scp -i $SSH_KEY keycloak.tar $SSH_TARGET:~'
                    sh 'scp -i $SSH_KEY magicdocs.tar $SSH_TARGET:~'

                    echo 'Loading Docker images on production server'
                    sh 'ssh -i $SSH_KEY $SSH_TARGET \'docker load -i pgvector.tar && \
                        docker load -i keycloak.tar && \
                        docker load -i magicdocs.tar\''

                    echo 'Running Docker containers on production server'
                    script {
                        sh "ssh -i $SSH_KEY $SSH_TARGET 'nbot run -f -n magicdocs \
                            -a server \
                                -i magicdocs:latest \
                                -p 3000 \
                                -e RUST_LOG=info \
                                -e RUST_BACKTRACE=0 \
                                -e DATABASE_URL=postgres://magicdocs:\\$MD_DB_PASS@db:5432/magicdocs \
                                -e KEYCLOAK_INTERNAL_ADDR=http://kc:8080 \
                                -e KEYCLOAK_EXTERNAL_ADDR=https://kc.treeleaf.dev \
                                -e KEYCLOAK_USER=admin \
                                -e KEYCLOAK_PASSWORD=\\$KEYCLOAK_ADMIN_PASSWORD \
                                -e KEYCLOAK_REALM=magicdocs \
                                -e KEYCLOAK_CLIENT=magicdocs \
                                -e KEYCLOAK_CLIENT_SECRET=\\$KEYCLOAK_CLIENT_SECRET \
                                -o docs.treeleaf.dev \
                                -m admin@treeleaf.dev \
                                --depends-on db \
                                --depends-on kc \
                            -a kc \
                                -i keycloak:latest \
                                -p 8080 \
                                -e KC_DB=postgres \
                                -e KC_DB_USERNAME=keycloak \
                                -e KC_DB_PASSWORD=\\$KC_DB_PASS \
                                -e KC_DB_URL_HOST=db \
                                -e KC_DB_URL_PORT=5432 \
                                -e KC_DB_URL_DATABASE=keycloak \
                                -o kc.treeleaf.dev \
                                -m admin@treeleaf.dev \
                                -c \"start --hostname=kc.treeleaf.dev --http-enabled=true --proxy-headers=xforwarded --health-enabled true\" \
                                --depends-on db \
                                --network-alias kc \
                            -a db \
                                -i pgvector:latest \
                                -v magicdocs_db:/var/lib/postgresql/data \
                                --network-alias db'"
                    }
                }
            }
        }
    }
    post {
        always {
            echo 'Cleaning up local environment'
            script {
                try {
                    sh 'docker rmi pgvector:latest keycloak:latest magicdocs:latest'
                } catch (Exception e) {
                    echo "Failed to remove Docker images - ${e.getMessage()}"
                }

                try {
                    sh 'docker network rm magicdocs_test_net'
                } catch (Exception e) {
                    echo "Failed to remove magicdocs_test_net - ${e.getMessage()}"
                }

                try {
                    sh 'rm pgvector.tar keycloak.tar magicdocs.tar'
                } catch (Exception e) {
                    echo "Failed to remove Docker image tarballs - ${e.getMessage()}"
                }

                sh 'docker image prune -f'
                sh 'docker volume prune -f'
            }

            echo 'Cleaning up production server'
            withCredentials(bindings: [sshUserPrivateKey(credentialsId: 'DeploymentTargetServer', keyFileVariable: 'SSH_KEY')]) {
                script {
                    try {
                        sh 'ssh -i $SSH_KEY $SSH_TARGET \'rm pgvector.tar keycloak.tar magicdocs.tar\''
                    } catch (Exception e) {
                        echo "Failed to remove Docker image tarballs on production server - ${e.getMessage()}"
                    }
                }
            }
        }
    }
}
