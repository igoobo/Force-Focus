import React from 'react';
import { GoogleLogin } from '@react-oauth/google';
import './login.css';
import useMainStore from '../../MainStore.jsx';
import logoIcon from '../layout/TitleBar/ForceFocus_icon.png'; 

const Login = ({ onLoginSuccess }) => {
    const isDarkMode = useMainStore((state) => state.isDarkMode);
    
    const handleGoogleSuccess = async (credentialResponse) => {
        try {
            // 1. 백엔드의 새로운 검증 엔드포인트로 ID 토큰 전송
            const response = await fetch('/api/v1/auth/google/verify', {
                method: 'POST',
                headers: {
                    'Content-Type': 'application/json',
                },
                body: JSON.stringify({
                    token: credentialResponse.credential // 구글에서 발행한 ID 토큰
                }),
            });

            if (response.ok) {
                const data = await response.json();
            
                // 2. 백엔드에서 자체적으로 생성하여 보낸 서비스 전용 토큰 저장
                localStorage.setItem('accessToken', data.access_token);
                localStorage.setItem('refreshToken', data.refresh_token);
            
                // 3. 메인 스토어 또는 상태 업데이트를 통한 로그인 처리
                onLoginSuccess(); 
            } else {
                const errorData = await response.json();
                alert(`로그인 실패: ${errorData.detail || '검증 오류'}`);
            }
        } catch (error) {
            console.error("Auth Error:", error);
            alert("서버와 통신 중 오류가 발생했습니다.");
        }
    };

    const handleGoogleError = () => {
        alert("구글 로그인에 실패했습니다. 다시 시도해 주세요.");
    };

    return (
        <div className={`login-container ${isDarkMode ? 'dark-theme' : ''}`}>
            <div className="login-wrapper"> {/* 두 카드를 세로로 정렬할 래퍼 */}
                
                {/* 첫 번째 행: 브랜드 카드 */}
                <div className="brand-card">
                    <img src={logoIcon} alt="ForceFocus Logo" className="login-brand-logo" />
                    <h1 className="brand-name"><br></br>Force-Focus <br></br> Web Dashboard</h1>
                </div>

                {/* 두 번째 행: 로그인 폼 카드 */}
                <div className="login-form-card">
                    <div className="login-header">
                        <h2>Dashboard Login</h2>
                        <p className="login-subtitle">서비스 이용을 위해 Google 계정으로 로그인해 주세요.</p>
                    </div>

                    <div className="google-login-wrapper">
                        <GoogleLogin
                            onSuccess={handleGoogleSuccess}
                            onError={handleGoogleError}
                            theme={isDarkMode ? "filled_black" : "outline"}
                            shape="pill"
                            size="large"
                            width="360px"
                            useOneTap
                        />
                    </div>

                    <div className="login-footer">
                        <p className="security-notice">
                            안전한 로그인을 위해 Google OAuth 2.0 시스템을 사용합니다.
                        </p>
                    </div>
                </div>
                
            </div>
        </div>
    );
};

export default Login;