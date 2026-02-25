import React, { useState } from "react";
import { GoogleLogin } from '@react-oauth/google';
import './login.css';
import useMainStore from '../../MainStore.jsx';
import logoIcon from '../layout/TitleBar/ForceFocus_icon.png'; 
import authApi from '../../api/authApi';

const Login = ({ onLoginSuccess }) => {
    const isDarkMode = useMainStore((state) => state.isDarkMode);
    const [isLoading, setIsLoading] = useState(false);
    
    const handleGoogleSuccess = async (credentialResponse) => {
        setIsLoading(true);
        try {
            // 백엔드 검증 엔드포인트 호출
            const response = await authApi.post('/api/v1/auth/google/verify', {
                token: credentialResponse.credential 
            });

            const data = response.data;
            const token = data.access_token; 

            if (token) {
                localStorage.setItem('accessToken', token);
                localStorage.setItem('refreshToken', data.refresh_token);
                onLoginSuccess(); 
            } else {
                throw new Error("Token not found in response");
            }   
        } catch (error) {
            console.error("Auth Network Error:", error);
            alert("서버와 통신 중 오류가 발생했습니다. 네트워크 상태를 확인해 주세요.");
            setIsLoading(false);
        }
    };

    const handleGoogleError = () => {
        alert("구글 로그인 세션이 만료되었거나 취소되었습니다. 다시 시도해 주세요.");
    };

    return (
        <div className={`login-container ${isDarkMode ? 'dark-theme' : ''}`}>
            <div className="login-wrapper">
                <div className="brand-card">
                    <img src={logoIcon} alt="ForceFocus Logo" className="login-brand-logo" />
                    <h1 className="brand-name"><br />Force-Focus <br /> Web Dashboard</h1>
                </div>

                <div className="login-form-card">
                    <div className="login-header">
                        <h2>Dashboard Login</h2>
                        <p className="login-subtitle">서비스 이용을 위해 Google 계정으로 로그인해 주세요.</p>
                    </div>

                    <div className="google-login-wrapper">
                        {isLoading ? (
                            // 로딩 중 표시될 UI
                            <div className="login-loading">
                                <div className="spinner"></div>
                                <p>로그인 정보를 확인 중입니다...</p>
                            </div>
                        ) : (
                            <GoogleLogin
                                onSuccess={handleGoogleSuccess}
                                onError={handleGoogleError}
                                theme={isDarkMode ? "filled_black" : "outline"}
                                shape="pill"
                                size="large"
                                width="360px"
                                useOneTap
                            />
                        )}
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