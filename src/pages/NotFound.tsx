import { useLocation } from "react-router-dom";
import { useEffect } from "react";

const NotFound = () => {
  const location = useLocation();

  useEffect(() => {
    console.error("404 Error: User attempted to access non-existent route:", location.pathname);
  }, [location.pathname]);

  return (
    <div className="flex min-h-screen items-center justify-center bg-background">
      <div className="text-center p-6">
        <img 
          src="/lovable-uploads/IMG_9255.png" 
          alt="AirADCR Logo" 
          className="h-16 w-auto mx-auto mb-6"
        />
        <h1 className="mb-4 text-4xl font-bold text-foreground">404</h1>
        <p className="mb-6 text-xl text-muted-foreground">Oops! Page non trouvée</p>
        <a 
          href="/" 
          className="inline-flex items-center justify-center px-6 py-2 bg-primary text-primary-foreground rounded-lg hover:bg-primary-hover transition-colors font-medium"
        >
          Retour à l'accueil
        </a>
      </div>
    </div>
  );
};

export default NotFound;
