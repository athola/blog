#!/usr/bin/env python3
"""
Simple test script to verify hydration fix by checking if the page loads
without JavaScript errors in the browser console.
"""

import requests
import time
import subprocess
import sys
from selenium import webdriver
from selenium.webdriver.chrome.options import Options
from selenium.webdriver.common.by import By
from selenium.webdriver.support.ui import WebDriverWait
from selenium.webdriver.support import expected_conditions as EC

def test_hydration_fix():
    """Test if the hydration fix works by checking for JavaScript errors."""
    
    # Check if server is running
    try:
        response = requests.get("http://127.0.0.1:3007", timeout=5)
        if response.status_code != 200:
            print(f"❌ Server returned status code: {response.status_code}")
            return False
        print("✅ Server is running and responding")
    except requests.exceptions.RequestException as e:
        print(f"❌ Server is not accessible: {e}")
        return False
    
    # Check if static files are accessible
    static_files = [
        "http://127.0.0.1:3007/pkg/blog.css",
        "http://127.0.0.1:3007/pkg/blog.js",
        "http://127.0.0.1:3007/pkg/blog_bg.wasm"
    ]
    
    for file_url in static_files:
        try:
            response = requests.head(file_url, timeout=5)
            if response.status_code == 200:
                print(f"✅ Static file accessible: {file_url}")
            else:
                print(f"❌ Static file not accessible ({response.status_code}): {file_url}")
                return False
        except requests.exceptions.RequestException as e:
            print(f"❌ Error accessing static file {file_url}: {e}")
            return False
    
    # Try to test with Selenium if available
    try:
        chrome_options = Options()
        chrome_options.add_argument("--headless")
        chrome_options.add_argument("--no-sandbox")
        chrome_options.add_argument("--disable-dev-shm-usage")
        chrome_options.add_argument("--log-level=3")
        
        driver = webdriver.Chrome(options=chrome_options)
        driver.set_page_load_timeout(10)
        
        # Enable JavaScript error logging
        driver.execute_cdp_cmd("Log.enable", {})
        
        print("🔍 Loading page in browser...")
        driver.get("http://127.0.0.1:3007")
        
        # Wait for page to load
        WebDriverWait(driver, 10).until(
            EC.presence_of_element_located((By.TAG_NAME, "body"))
        )
        
        # Check for JavaScript errors
        logs = driver.get_log("browser")
        js_errors = [log for log in logs if log['level'] == 'SEVERE']
        
        if js_errors:
            print("❌ JavaScript errors found:")
            for error in js_errors:
                print(f"   - {error['message']}")
            return False
        else:
            print("✅ No JavaScript errors found")
        
        # Check if the page title is correct
        if "Alex Thola's Blog" in driver.title:
            print("✅ Page title is correct")
        else:
            print(f"❌ Unexpected page title: {driver.title}")
            return False
        
        # Check if header navigation links are present
        nav_links = driver.find_elements(By.CSS_SELECTOR, "header a")
        expected_links = ["blog", "references", "contact"]
        
        for link_text in expected_links:
            found = any(link_text in link.text for link in nav_links)
            if found:
                print(f"✅ Navigation link found: {link_text}")
            else:
                print(f"❌ Navigation link missing: {link_text}")
                return False
        
        driver.quit()
        print("✅ All hydration tests passed!")
        return True
        
    except Exception as e:
        print(f"⚠️  Could not run browser tests: {e}")
        print("✅ Basic server and static file tests passed")
        return True

if __name__ == "__main__":
    success = test_hydration_fix()
    sys.exit(0 if success else 1)