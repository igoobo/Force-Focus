import React, { useState } from 'react';
import { GoogleLogin } from '@react-oauth/google'; // GoogleLogin 컴포넌트 추가
import './login.css';

const Login = ({ onLoginSuccess }) => {
    const [credentials, setCredentials] = useState({ username: '', password: '' });
    const [isLoading, setIsLoading] = useState(false);

    const handleChange = (e) => {
        const { name, value } = e.target;
        setCredentials({ ...credentials, [name]: value });
    };

    const handleSubmit = (e) => {
        e.preventDefault();
        setIsLoading(true);

        // 일반 로그인 모의 처리
        setTimeout(() => {
            localStorage.setItem('accessToken', 'test-token-12345');
            setIsLoading(false);
            onLoginSuccess(); // App.jsx의 isLoggedIn을 true로 변경
        }, 1000);
    };

    // 구글 로그인 성공 핸들러
    const handleGoogleSuccess = (credentialResponse) => {
        console.log("Google Login Success, Token:", credentialResponse.credential);
        
        // 1. 구글에서 받은 토큰을 로컬 스토리지에 저장
        localStorage.setItem('accessToken', credentialResponse.credential);
        
        // 2. 부모(App.jsx)에게 로그인 성공을 알림
        // 이 함수가 호출되면 App.jsx의 isLoggedIn이 true가 되어 대시보드 렌더링 수행
        onLoginSuccess();
    };

    const handleGoogleError = () => {
        alert("구글 로그인에 실패했습니다. 다시 시도해 주세요.");
    };

    return (
        <div className="login-container">
            <form onSubmit={handleSubmit} className="login-form">
                <h2>Dashboard Login</h2>
                
                <div className="input-group">
                    <label htmlFor="username">Username</label>
                    <input
                        id="username"
                        type="text"
                        name="username"
                        value={credentials.username}
                        onChange={handleChange}
                        required
                        placeholder="아이디를 입력하세요"
                    />
                </div>
                
                <div className="input-group">
                    <label htmlFor="password">Password</label>
                    <input
                        id="password"
                        type="password"
                        name="password"
                        value={credentials.password}
                        onChange={handleChange}
                        required
                        placeholder="비밀번호를 입력하세요"
                    />
                </div>

                <button type="submit" className={`login-btn ${isLoading ? 'loading' : ''}`} disabled={isLoading}>
                    {isLoading ? <div className="spinner"></div> : '로그인'}
                </button>

                <div className="divider-container">
                    <span className="divider-line"></span>
                    <span className="divider-text">OR</span>
                    <span className="divider-line"></span>
                </div>

                {/* 구글 로그인 버튼: 기존 커스텀 버튼 대신 공식 라이브러리 컴포넌트 사용 */}
                <div style={{ display: 'flex', justifyContent: 'center', marginTop: '10px' }}>
                    <GoogleLogin
                        onSuccess={handleGoogleSuccess}
                        onError={handleGoogleError}
                        useOneTap
                    />
                </div>

                <div className="login-footer">
                    <a href="#find">비밀번호 찾기</a>
                    <span className="footer-divider">|</span>
                    <a href="#signup">회원가입</a>
                </div>
            </form>
        </div>
    );
};

export default Login;