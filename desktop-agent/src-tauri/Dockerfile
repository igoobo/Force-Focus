# ----------------------------------------------------
# 1. 베이스 이미지 선택: Tauri 빌드에 필요한 최소한의 시스템 의존성이 포함된 Ubuntu 이미지를 사용합니다.
#    Windows용 Tauri 앱을 빌드하더라도, 빌드 환경 자체는 Linux 컨테이너 안에서 만듭니다.
FROM ubuntu:22.04

# ----------------------------------------------------
# 2. 시스템 의존성 설치:
#    Tauri 빌드에 필요한 Linux 라이브러리들 (주로 Tauri 빌드 툴체인 자체 또는 Linux 타겟용).
RUN apt-get update && apt-get install -y \
    build-essential \
    curl \
    wget \
    file \
    libssl-dev \
    libgtk-3-dev \
    libwebkit2gtk-4.0-dev \
    librsvg2-dev \
    libappindicator3-dev \
    patchelf \
    libvips \
    pkg-config \
    zip \
    && rm -rf /var/lib/apt/lists/*

# ----------------------------------------------------
# 3. Node.js 설치: 웹 프론트엔드를 빌드하기 위해 Node.js를 설치합니다.
RUN curl -sL https://deb.nodesource.com/setup_18.x | bash -
RUN apt-get install -y nodejs

# ----------------------------------------------------
# 4. Rust 설치: Tauri 앱의 핵심인 Rust 툴체인을 설치합니다.
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

# ----------------------------------------------------
# 5. Rust Windows 크로스-컴파일 타겟 추가:
#    이 Dockerfile의 핵심 목적 중 하나는 Windows용 Tauri 앱을 빌드하는 것이므로,
#    Rust에게 Windows 타겟을 추가하도록 지시합니다.
#    MSVC(Microsoft Visual C++) 툴체인을 사용하는 x86_64 아키텍처 Windows 타겟입니다.
RUN rustup target add x86_64-pc-windows-msvc

# ----------------------------------------------------
# 6. 작업 디렉토리 설정: 컨테이너 내부의 /app 디렉토리를 작업 공간으로 지정합니다.
WORKDIR /app

# ----------------------------------------------------
# 7. 의존성 캐싱 (Node.js):
#    package.json과 package-lock.json 파일을 먼저 복사하여 npm 의존성을 설치합니다.
COPY package*.json ./
RUN npm install

# ----------------------------------------------------
# 8. Rust 의존성 캐싱:
#    Cargo.toml과 Cargo.lock 파일을 먼저 복사하여 Rust 의존성을 미리 다운로드합니다.
RUN mkdir -p src-tauri
COPY src-tauri/Cargo.toml src-tauri/
RUN cd src-tauri && cargo fetch

# ----------------------------------------------------
# 9. 애플리케이션 코드 복사:
#    로컬 디렉토리(desktop-agent/)의 나머지 모든 소스 코드를 컨테이너의 /app 디렉토리로 복사합니다.
COPY . .

# ----------------------------------------------------
# 10. 빌드 명령 (실행 시 사용):
#     이 Dockerfile로 만들어진 이미지로 컨테이너를 실행할 때,
#     어떤 명령을 사용해야 Windows용 Tauri 앱을 빌드할 수 있는지 예시입니다.
#     CI/CD 파이프라인에서 이 이미지를 사용하여 다음 명령을 실행하게 됩니다:
#     `npm run tauri build -- --target x86_64-pc-windows-msvc`
#     이 명령어는 `desktop-agent/package.json`에 정의된 `tauri build` 스크립트를 실행하며,
#     Rust에게 Windows 타겟용으로 빌드하라고 지시합니다.
CMD ["/bin/bash"] # 컨테이너 진입 시 bash 쉘을 시작합니다 (빌드 명령은 외부에서).