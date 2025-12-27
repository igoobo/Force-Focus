import React from 'react';
import { GoogleLogin } from '@react-oauth/google';
import './login.css';

const Login = ({ onLoginSuccess }) => {
    
    // 구글 로그인 성공 핸들러
    const handleGoogleSuccess = (credentialResponse) => {
        console.log("Google Login Success, Token:", credentialResponse.credential);
        
        // 1. 구글에서 받은 토큰을 로컬 스토리지에 저장
        localStorage.setItem('accessToken', credentialResponse.credential);
        
        // 2. 부모(App.jsx)에게 로그인 성공을 알림
        onLoginSuccess();
    };

    const handleGoogleError = () => {
        console.log("Google Login Failed");
        alert("구글 로그인에 실패했습니다. 다시 시도해 주세요.");
    };

    return (
        <div className="login-container">
            <form className="login-form">
                <div className="login-logo">
                    <h2>Dashboard Login</h2>
                    <p>서비스 이용을 위해 Google 계정으로 로그인해 주세요.</p>
                </div>

                {/* 구글 로그인 버튼만 단독 배치 */}
                <div style={{ 
                    display: 'flex', 
                    justifyContent: 'center', 
                    marginTop: '30px',
                    marginBottom: '20px' 
                }}>
                    <GoogleLogin
                        onSuccess={handleGoogleSuccess}
                        onError={handleGoogleError}
                        useOneTap
                    />
                </div>

                <div className="login-footer">
                    <p style={{ fontSize: '12px', color: '#888' }}>
                        안전한 로그인을 위해 Google OAuth 2.0 시스템을 사용합니다.
                    </p>
                </div>
            </form>
        </div>
    );
};

export default Login;