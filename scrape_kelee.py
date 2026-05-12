# /// script
# requires-python = ">=3.10"
# dependencies = [
#     "pydoll-python",
#     "beautifulsoup4",
# ]
# ///

import asyncio
import time
from pydoll.browser import Chrome
from pydoll.browser.options import ChromiumOptions
from bs4 import BeautifulSoup
import urllib.parse

async def scrape_urls(url: str, headless: bool = True):
    options = ChromiumOptions()
    options.headless = headless
    options.binary_location = '/Applications/Google Chrome Dev.app/Contents/MacOS/Google Chrome Dev'

    fake_engagement_time = int(time.time()) - (7 * 24 * 60 * 60)
    options.browser_preferences = {
        'profile': {
            'last_engagement_time': fake_engagement_time,
            'exit_type': 'Normal',
            'exited_cleanly': True,
        },
    }
    options.webrtc_leak_protection = True

    async with Chrome(options=options) as browser:
        tab = await browser.start()

        print("Starting Cloudflare bypass...")
        async with tab.expect_and_bypass_cloudflare_captcha():
            await tab.go_to(url)

        print("Waiting for page to load completely...")
        await asyncio.sleep(5)
        
        html = await tab.page_source
        
        soup = BeautifulSoup(html, 'html.parser')
        
        links = []
        for a_tag in soup.find_all('a', href=True):
            href = a_tag['href']
            full_url = urllib.parse.urljoin(url, href)
            if full_url.startswith('http'):
                links.append(full_url)
                
        return set(links)

async def main():
    print("Scraping URLs from https://kelee.one/...")
    urls = await scrape_urls('https://kelee.one/')
    print(f"Found {len(urls)} unique URLs:")
    for u in sorted(urls):
        print(f"  - {u}")

if __name__ == '__main__':
    asyncio.run(main())